use std::{cell::RefCell, collections::HashMap, rc::Rc};

use instruction::{temp::TempManager, LlvmInstrSet, RiscvInstrSet};
use llvm::llvminstr::LlvmInstr;

use crate::transformer::to_riscv;
use utils::errors::Result;

type Node = Rc<RefCell<InstrNode>>;

pub struct InstrNode {
	pub in_deg: usize,
	pub instr: RiscvInstrSet,
	pub succ: Vec<Node>,
}

pub struct InstrDag {
	pub nodes: Vec<Node>,
}

impl InstrNode {
	pub fn new(
		instr: &LlvmInstr,
		succ: Vec<Node>,
		mgr: &mut TempManager,
	) -> Result<InstrNode> {
		Ok(InstrNode {
			in_deg: instr.get_read().len(),
			instr: to_riscv(instr, mgr)?,
			succ,
		})
	}
}

impl InstrDag {
	pub fn new(instrs: &LlvmInstrSet, mgr: &mut TempManager) -> Result<InstrDag> {
		let mut nodes = Vec::new();
		let mut edge = HashMap::new();
		for instr in instrs.iter().rev() {
			let prev = instr.get_read();
			let node = if let Some(target) = instr.get_write() {
				InstrNode::new(instr, edge.remove(&target).unwrap_or(Vec::new()), mgr)
			} else {
				InstrNode::new(instr, Vec::new(), mgr)
			}?;
			let node = Rc::new(RefCell::new(node));
			for label in prev {
				edge.entry(label).or_insert_with(Vec::new).push(Rc::clone(&node));
			}
			nodes.push(node);
		}
		Ok(InstrDag { nodes })
	}
}
