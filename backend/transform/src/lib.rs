use std::{cell::RefCell, collections::HashMap, rc::Rc};

use instr_dag::InstrDag;
use instruction::{
	riscv::{
		convert::i32_to_reg,
		reg::{
			RiscvReg::{SP, X0},
			PARAMETER_REGS,
		},
		riscvinstr::{ITriInstr, RTriInstr},
		riscvop::{ITriInstrOp::Addi, RTriInstrOp::Add},
		value::is_lower,
	},
	temp::TempManager,
};

use rrvm::{
	cfg::{link_node, BasicBlock},
	program::*,
	LlvmNode, RiscvCFG, RiscvNode,
};
use transformer::to_riscv;
use utils::errors::Result;

pub mod instr_dag;
pub mod instr_schedule;
pub mod remove_phi;
pub mod transformer;

use crate::instr_schedule::instr_schedule;

pub fn convert_func(func: LlvmFunc) -> Result<RiscvFunc> {
	let mut nodes = Vec::new();
	let mgr = &mut TempManager::new(0);
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
				let num = i32_to_reg(kill_size, &mut node.borrow_mut().instrs, mgr);
				RTriInstr::new(Add, SP.into(), SP.into(), num)
			};
			node.borrow_mut().instrs.push(instr);
		}
		let jump = to_riscv(block.borrow().jump_instr.as_ref().unwrap(), mgr)?;
		node.borrow_mut().instrs.extend(jump);
		nodes.push(node);
	}
	for (u, v) in edge {
		link_node(table.get(&u).unwrap(), table.get(&v).unwrap())
	}
	for node in nodes.iter() {
		let instr = node.borrow_mut().instrs.pop().unwrap();
		node.borrow_mut().set_jump(Some(instr));
	}

	for (temp, reg) in func.params.iter().zip(PARAMETER_REGS.iter()).rev() {
		let reg = mgr.new_pre_color_temp(*reg);
		let temp = mgr.get(&temp.into());
		let instr = RTriInstr::new(Add, temp, reg, X0.into());
		nodes.first().unwrap().borrow_mut().instrs.insert(0, instr);
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
	block.kill_size = node.borrow().kill_size;
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
	block.kill_size = node.borrow().kill_size;
	block.instrs = instrs?.into_iter().flatten().collect();
	Ok(Rc::new(RefCell::new(block)))
}
