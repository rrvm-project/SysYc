use std::{cell::RefCell, collections::HashMap, rc::Rc};

use instr_dag::InstrDag;
use instruction::{riscv::prelude::*, temp::TempManager};

use llvm::Value;
use rrvm::prelude::*;
use transformer::to_riscv;
use utils::{errors::Result, SysycError::RiscvGenError};

pub mod instr_dag;
pub mod instr_schedule;
pub mod remove_phi;
pub mod transformer;

use crate::instr_schedule::instr_schedule;

pub fn get_functions(
	program: &mut RiscvProgram,
	funcs: Vec<LlvmFunc>,
) -> Result<()> {
	for func in funcs {
		program.funcs.push(convert_func(func, &mut program.temp_mgr)?);
	}
	Ok(())
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

	Ok(RiscvFunc {
		total: mgr.total,
		spills: 0,
		cfg: RiscvCFG { blocks: nodes },
		name: func.name,
		params: func.params,
		ret_type: func.ret_type,
	})
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
