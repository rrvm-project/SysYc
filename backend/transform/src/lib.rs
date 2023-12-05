use std::{cell::RefCell, collections::HashMap, rc::Rc};

use instr_dag::InstrDag;
use instruction::{riscv::riscvinstr::RiscvInstr, temp::TempManager};
use llvm::LlvmInstr;
use rrvm::{
	basicblock::Node,
	cfg::{link_node, BasicBlock},
	program::*,
	RiscvCFG,
};
use transformer::to_riscv;
use utils::errors::Result;

pub mod instr_dag;
pub mod instr_schedule;
pub mod remove_phi;
pub mod transformer;

use crate::instr_schedule::instr_schedule;

pub fn convert_func(func: LlvmFunc) -> Result<RiscvFunc> {
	let mut blocks = Vec::new();
	let mgr = &mut TempManager::new();
	let mut edge = Vec::new();
	let mut table = HashMap::new();
	func.cfg.blocks.iter().for_each(remove_phi::remove_phi);
	for u in func.cfg.blocks {
		let u_id = u.borrow().id;
		edge.extend(u.borrow().succ.iter().map(|v| (u_id, v.borrow().id)));
		let block = transform_basicblock(u, mgr)?;
		table.insert(u_id, block.clone());
		blocks.push(block)
	}
	for (u, v) in edge {
		link_node(table.get(&u).unwrap(), table.get(&v).unwrap())
	}
	Ok(RiscvFunc {
		cfg: RiscvCFG { blocks },
		name: func.name,
		params: func.params,
		ret_type: func.ret_type,
	})
}

pub fn transform_basicblock(
	node: Node<LlvmInstr>,
	mgr: &mut TempManager,
) -> Result<Node<RiscvInstr>> {
	let instr_dag = InstrDag::new(&node.borrow().instrs, mgr)?;
	let mut block = BasicBlock::new(node.borrow().id);
	block.instrs = instr_schedule(instr_dag)?;
	block
		.instrs
		.extend(to_riscv(node.borrow().jump_instr.as_ref().unwrap(), mgr)?);
	Ok(Rc::new(RefCell::new(block)))
}
