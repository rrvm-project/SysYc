use std::{
	cell::RefCell,
	collections::{HashMap, HashSet},
	rc::Rc,
};

use instruction::{riscv::prelude::*, temp::TempManager};
use rrvm::prelude::*;
use transformer::{to_riscv, to_rt_type};
use utils::{errors::Result, BLOCKSIZE_THRESHOLD};

pub mod remove_phi;
pub mod transformer;

pub fn get_functions(
	program: &mut RiscvProgram,
	funcs: Vec<LlvmFunc>,
) -> Result<()> {
	for func in funcs {
		let converted_func = convert_func(func, &mut program.temp_mgr)?;
		program.funcs.push(instr_schedule(converted_func, &mut program.temp_mgr)?);
	}
	Ok(())
}

pub fn instr_schedule(
	func: RiscvFunc,
	mgr: &mut TempManager,
) -> Result<RiscvFunc> {
	func.cfg.clear_data_flow();
	func.cfg.analysis();
	let mut new_blocks = Vec::new();
	for node in func.cfg.blocks.iter() {
		let nodes = instr_schedule_block(node, mgr)?;
		new_blocks.extend(nodes);
	}
	Ok(RiscvFunc {
		total: mgr.total,
		spills: 0,
		cfg: RiscvCFG { blocks: new_blocks },
		name: func.name,
		params: func.params,
		ret_type: func.ret_type,
		external_resorce: HashSet::new(),
	})
}
pub fn instr_schedule_block(
	riscv_node: &RiscvNode,
	mgr: &mut TempManager,
) -> Result<Vec<RiscvNode>> {
	let prev = riscv_node
		.borrow()
		.prev
		.iter()
		.map(|v| v.borrow().id)
		.collect::<HashSet<_>>();
	let succ = riscv_node
		.borrow()
		.succ
		.iter()
		.map(|v| v.borrow().id)
		.collect::<HashSet<_>>();
	// 判断 prev 和 succ 是否有交集
	if prev.intersection(&succ).count() > 0
		&& riscv_node.borrow().instrs.len() <= BLOCKSIZE_THRESHOLD
	{
		// filter call (instrs 中不能有 call 指令)
		let mut has_call = false;
		for instr in riscv_node.borrow().instrs.iter() {
			if instr.is_call() {
				has_call = true;
				break;
			}
		}
		if (!has_call) {
			return transform_loop_block(&riscv_node, mgr, 4);
		} else {
			match transform_basic_block_by_pipelining(&riscv_node, mgr) {
				Ok(v) => Ok(vec![v]),
				Err(e) => Err(e),
			}
		}
	} else {
		match transform_basic_block_by_pipelining(&riscv_node, mgr) {
			Ok(v) => Ok(vec![v]),
			Err(e) => Err(e),
		}
	}
}
pub fn convert_func(
	func: LlvmFunc,
	mgr: &mut TempManager,
) -> Result<RiscvFunc> {
	let mut nodes = Vec::new();
	let mut edge = Vec::new();
	let mut table = HashMap::new();
	let mut alloc_table = HashMap::new();
	func.cfg.blocks.iter().for_each(remove_phi::remove_phi);
	for block in func.cfg.blocks.iter() {
		for instr in block.borrow().instrs.iter() {
			if let Some((temp, length)) = instr.get_alloc() {
				alloc_table.insert(temp, length);
			}
		}
	}

	for block in func.cfg.blocks {
		let kill_size = (block.borrow().kill_size + 15) & -16;
		let id = block.borrow().id;
		edge.extend(block.borrow().succ.iter().map(|v| (id, v.borrow().id)));
		let node = transform_basicblock(&block, mgr)?;
		table.insert(id, node.clone());
		if kill_size != 0 {
			let instr = if is_lower(kill_size) {
				ITriInstr::new(Addi, SP.into(), SP.into(), kill_size.into())
			} else {
				let num = load_imm(kill_size, &mut node.borrow_mut().instrs, mgr);
				RTriInstr::new(Add, SP.into(), SP.into(), num)
			};
			node.borrow_mut().instrs.push(instr);
		}
		let mut instrs =
			to_riscv(block.borrow().jump_instr.as_ref().unwrap(), mgr)?;
		node.borrow_mut().set_jump(instrs.pop());
		node.borrow_mut().instrs.extend(instrs);
		nodes.push(node);
	}
	for (u, v) in edge {
		force_link_node(table.get(&u).unwrap(), table.get(&v).unwrap())
	}

	Ok(RiscvFunc {
		total: mgr.total,
		spills: 0,
		cfg: RiscvCFG { blocks: nodes },
		name: func.name,
		params: func.params,
		ret_type: func.ret_type,
		external_resorce: HashSet::new(),
		entrance: Entrance::Unkonwn,
	})
}

fn transform_loop_block(
	node: &RiscvNode,
	mgr: &mut TempManager,
	n: usize, // 展开次数
) -> Result<Vec<RiscvNode>> {
	// calc T_0
	let R = [1, 1, 1, 1, 2]; // mem,br,mul/div,floating-point,sum
												 //按照RT 求出总的资源占用，再和 R 中各项相除求得最大值
	let mut rt = [0, 0, 0, 0, 0];
	for instr in node.borrow().instrs.iter() {
		let rt_vec = to_rt_type(instr);
		for i in 0..5 {
			rt[i] += rt_vec[i];
		}
	}
	let mut t0 = 0;
	for i in 0..5 {
		t0 = t0.max((rt[i] + R[i] - 1) / R[i]);
	}
	// 模数变量扩展
	// 找到本循环内 def 且 use 非 live_in 非 live_out 的变量
	let mut tmps = HashSet::new();
	for tmp in node.borrow().defs.intersection(&node.borrow().uses) {
		if !node.borrow().live_in.contains(tmp)
			&& !node.borrow().live_out.contains(tmp)
		{
			tmps.insert(*tmp);
		}
	}
	// 建立数据依赖图
	//let mut dag = HashMap::new();
	// 先加上非数组的边
	for (idx, instr) in node.borrow().instrs.iter().enumerate() {
		let read_tmps = instr.get_riscv_read();
		for i in read_tmps.iter() {
			// 找上一次 write 的地方
			// 先往该基本块,该指令前面的若干指令找write
			let mut find = false;
			let mut alpha = 0;
		}
	}
	Err(utils::SysycError::RiscvGenError(
		"Loop block not supported".to_string(),
	))
}
fn transform_basic_block_by_pipelining(
	node: &RiscvNode,
	mgr: &mut TempManager,
) -> Result<RiscvNode> {
	todo!()
}
fn transform_basicblock(
	node: &LlvmNode,
	mgr: &mut TempManager,
) -> Result<RiscvNode> {
	// 先识别该基本块是否是基本本块（循环内只有一个基本块的情况），判断其前驱后继是否含有同一个基本块
	let instrs: Result<Vec<_>, _> =
		node.borrow().instrs.iter().map(|v| to_riscv(v, mgr)).collect();
	let mut block = BasicBlock::new(node.borrow().id, node.borrow().weight);
	block.kill_size = node.borrow().kill_size;
	block.instrs = instrs?.into_iter().flatten().collect();
	let riscv_node = Rc::new(RefCell::new(block));
	Ok(riscv_node)
}
