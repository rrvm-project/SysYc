use std::{cell::RefCell, collections::HashMap, rc::Rc};

use instr_dag::InstrDag;
use instruction::{riscv::prelude::*, temp::TempManager};

use llvm::Value;
use la_reduce::la_reduce_func;
use rrvm::prelude::*;
use transformer::to_riscv;
use utils::{errors::Result, SysycError::RiscvGenError};

pub mod instr_dag;
pub mod instr_schedule;
pub mod la_reduce;
pub mod remove_phi;
pub mod transformer;
use crate::instr_schedule::instr_schedule;

pub fn get_functions(
	program: &mut RiscvProgram,
	funcs: Vec<LlvmFunc>,
) -> Result<()> {
	let mut pcrel_mgr = PCRelMgr::new();
	for func in funcs {
		let (mut myfunc, liveins, liveouts) =
			convert_func(func, &mut program.temp_mgr)?;
		// println!("func instrs:");
		// for block in myfunc.cfg.blocks.iter(){
		// 	for instr in block.borrow().instrs.iter(){
		// 		println!("{}",instr);
		// 	}
		// }
		la_reduce_func(&mut myfunc, liveouts, &mut pcrel_mgr);
		program.funcs.push(myfunc);
	}
	Ok(())
}

#[allow(clippy::type_complexity)]
pub fn convert_func(
	func: LlvmFunc,
	mgr: &mut TempManager,
) -> Result<(RiscvFunc, Vec<HashSet<RiscvTemp>>, Vec<HashSet<RiscvTemp>>)> {
	let mut nodes = Vec::new();
	let mut edge = Vec::new();
	let mut table = HashMap::new();
	let mut alloc_table = HashMap::new();
	let mut live_ins = Vec::new();
	let mut live_outs = Vec::new();
	func.cfg.blocks.iter().for_each(remove_phi::remove_phi);
	// debug print
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

	for block in func.cfg.blocks.iter() {
		let live_in: HashSet<_> =
			block.borrow().live_in.iter().map(|v| mgr.get(v)).collect();
		let live_out: HashSet<_> =
			block.borrow().live_out.iter().map(|v| mgr.get(v)).collect();
		live_ins.push(live_in);
		live_outs.push(live_out);
	}
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
			external_resorce: HashSet::new(),
		},
		live_ins,
		live_outs,
	))
}
fn _transform_basicblock_by_dag(
	node: &LlvmNode,
	mgr: &mut TempManager,
) -> Result<RiscvNode> {
	let instr_dag = InstrDag::new(&node.borrow().instrs, mgr)?;
	let mut block = BasicBlock::new(node.borrow().id, node.borrow().weight);
	block.instrs = instr_schedule(instr_dag)?;
	Ok(Rc::new(RefCell::new(block)))
}

fn transform_basicblock(
	node: &LlvmNode,
	mgr: &mut TempManager,
) -> Result<RiscvNode> {
	let instrs: Result<Vec<_>, _> =
		node.borrow().instrs.iter().map(|v| to_riscv(v, mgr)).collect();
	let mut block = BasicBlock::new(node.borrow().id, node.borrow().weight);
	block.instrs = instrs?.into_iter().flatten().collect();
	Ok(Rc::new(RefCell::new(block)))
}
