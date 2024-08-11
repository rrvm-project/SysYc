use std::{
	cell::RefCell,
	collections::{BTreeMap, HashMap, HashSet},
	io::{self, Write},
	rc::Rc,
};

use instr_schedule::instr_schedule_by_dag;
use instrdag::InstrDag;
use instruction::{riscv::prelude::*, temp::TempManager};

use llvm::Value;
use utils::{SysycError::RiscvGenError};
use rrvm::prelude::*;
use transformer::{to_riscv, to_rt_type};
use utils::{
	errors::Result, BLOCKSIZE_THRESHOLD, DEPENDENCY_EXPLORE_DEPTH,
	SCHEDULE_THRESHOLD,
};

pub mod instr_schedule;
pub mod instrdag;
pub mod remove_phi;
pub mod transformer;

pub fn get_functions(
	program: &mut RiscvProgram,
	funcs: Vec<LlvmFunc>,
) -> Result<()> {
	for func in funcs {
		let converted_func = convert_func(func, &mut program.temp_mgr)?;
		println!("--- before instr schedule: ---");
		for i in converted_func.0.cfg.blocks.iter() {
			for j in i.borrow().instrs.iter() {
				println!("{}", j);
			}
			println!("------------block end-------------");
			// println!(
			// 	"jump instruction: {}",
			// 	i.borrow().jump_instr.as_ref().unwrap()
			// );
		}
		println!("---end---");
		io::stdout().flush().unwrap();
		let func = instr_schedule(
			converted_func.0,
			converted_func.1,
			converted_func.2,
			&mut program.temp_mgr,
		)?;
		println!("--------");
		for i in func.cfg.blocks.iter() {
			for j in i.borrow().instrs.iter() {
				println!("{}", j);
			}
			println!("------------block end-------------");
		}
		println!("--------");
		program.funcs.push(func);
	}
	Ok(())
}

pub fn instr_schedule(
	func: RiscvFunc,
	live_ins: Vec<HashSet<RiscvTemp>>,
	live_outs: Vec<HashSet<RiscvTemp>>,
	mgr: &mut TempManager,
) -> Result<RiscvFunc> {
	func.cfg.clear_data_flow();
	func.cfg.analysis();
	let mut new_blocks = Vec::new();
	for (idx, node) in func.cfg.blocks.iter().enumerate() {
		let nodes =
			instr_schedule_block(node, &live_ins[idx], &live_outs[idx], mgr)?;
		new_blocks.extend(nodes);
	}
	Ok(RiscvFunc {
		total: mgr.total,
		spills: 0,
		cfg: RiscvCFG { blocks: new_blocks },
		name: func.name,
		params: func.params,
		ret_type: func.ret_type,
	})
}
pub fn instr_schedule_block(
	riscv_node: &RiscvNode,
	live_ins: &HashSet<RiscvTemp>,
	live_outs: &HashSet<RiscvTemp>,
	mgr: &mut TempManager,
) -> Result<Vec<RiscvNode>> {
	if riscv_node.borrow().instrs.len() >= SCHEDULE_THRESHOLD {
		return Ok(vec![riscv_node.clone()]);
	}
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
		if riscv_node.borrow().instrs.iter().any(|instr| instr.is_call()) {
			transform_basic_block_by_pipelining(riscv_node, live_ins, live_outs, mgr)
				.map(|v| vec![v])
		} else {
			transform_basic_block_by_pipelining(riscv_node, live_ins, live_outs, mgr)
				.map(|v| vec![v])
		}
	} else {
		transform_basic_block_by_pipelining(riscv_node, live_ins, live_outs, mgr)
			.map(|v| vec![v])
	}
}
pub fn convert_func(
	func: LlvmFunc,
	mgr: &mut TempManager,
) -> Result<(RiscvFunc, Vec<HashSet<RiscvTemp>>, Vec<HashSet<RiscvTemp>>)> {
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

	let mut kill_size = 0;
	for block in func.cfg.blocks.iter() {
		for instr in block.borrow().instrs.iter() {
			if let Some((_, length)) = instr.get_alloc() {
				if let Value::Int(length) = length {
					kill_size += length;
				} else {
					return Err(RiscvGenError("Invalid alloc length".to_string()));
				}
			}
		}
	}
	kill_size = (kill_size + 15) & -16;

	for block in func.cfg.blocks {
		let id = block.borrow().id;
		edge.extend(block.borrow().succ.iter().map(|v| (id, v.borrow().id)));
		let node = transform_basicblock(&block, mgr)?;
		table.insert(id, node.clone());
		if kill_size != 0 && block.borrow().jump_instr.as_ref().unwrap().is_ret() {
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
	Ok((
		RiscvFunc {
			total: mgr.total,
			spills: 0,
			cfg: RiscvCFG { blocks: nodes },
			name: func.name,
			params: func.params,
			ret_type: func.ret_type,
		},
		live_ins,
		live_outs,
	))
}

fn transform_basic_block_by_pipelining(
	node: &RiscvNode,
	live_in: &HashSet<RiscvTemp>,
	live_out: &HashSet<RiscvTemp>,
	_mgr: &mut TempManager,
) -> Result<RiscvNode> {
	let mut instr_dag = InstrDag::new(node)?;
	let liveliness_map = get_liveliness_map(&instr_dag, live_in, live_out);
	instr_dag.assign_nodes();
	node.borrow_mut().instrs = instr_schedule_by_dag(instr_dag, liveliness_map)?;
	Ok(node.clone())
}
#[derive(Clone, Debug)]
pub struct Liveliness {
	is_livein: bool,
	is_liveout: bool,
	use_num: usize,
}
fn get_liveliness_map(
	node: &InstrDag,
	live_in: &HashSet<RiscvTemp>,
	live_out: &HashSet<RiscvTemp>,
) -> HashMap<RiscvTemp, Liveliness> {
	let mut map = HashMap::new();
	let mut call_reads = node.call_reads.clone();
	call_reads.reverse();
	let mut call_writes = node.call_writes.clone();
	call_writes.reverse();
	// 它这里要求是正序遍历，所以遍历次序是和 node 的顺序反的，需要 iter.rev(),同样，call_reads,call_writes 也要reverse再pop
	for instrnode in node.nodes.iter().rev() {
		let instr = &instrnode.borrow().instr;
		if !instr.is_call() {
			for tmp in instr.get_riscv_read().iter() {
				map
					.entry(*tmp)
					.or_insert(Liveliness {
						is_livein: false,
						is_liveout: false,
						use_num: 0,
					})
					.use_num += 1;
			}
			for tmp in instr.get_riscv_write().iter() {
				map.entry(*tmp).or_insert(Liveliness {
					is_livein: false,
					is_liveout: false,
					use_num: 0,
				});
			}
		} else {
			let call_read = call_reads.pop().unwrap();
			for tmp in call_read.iter() {
				map
					.entry(*tmp)
					.or_insert(Liveliness {
						is_livein: false,
						is_liveout: false,
						use_num: 0,
					})
					.use_num += 1;
			}
			let call_write = call_writes.pop().unwrap();
			for tmp in call_write.iter() {
				map.entry(*tmp).or_insert(Liveliness {
					is_livein: false,
					is_liveout: false,
					use_num: 0,
				});
			}
		}
	}
	// do live_in
	for tmp in live_in.iter() {
		map
			.entry(*tmp)
			.or_insert(Liveliness {
				is_livein: true,
				is_liveout: false,
				use_num: 0,
			})
			.is_livein = true;
	}
	for tmp in live_out.iter() {
		map
			.entry(*tmp)
			.or_insert(Liveliness {
				is_livein: false,
				is_liveout: true,
				use_num: 0,
			})
			.is_liveout = true;
	}
	map
}
fn transform_basicblock(
	node: &LlvmNode,
	mgr: &mut TempManager,
) -> Result<RiscvNode> {
	// 先识别该基本块是否是基本本块（循环内只有一个基本块的情况），判断其前驱后继是否含有同一个基本块
	let instrs: Result<Vec<_>, _> =
		node.borrow().instrs.iter().map(|v| to_riscv(v, mgr)).collect();
	let mut block = BasicBlock::new(node.borrow().id, node.borrow().weight);
	block.instrs = instrs?.into_iter().flatten().collect();
	let riscv_node = Rc::new(RefCell::new(block));
	Ok(riscv_node)
}
