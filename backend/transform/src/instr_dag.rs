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
	pub fn new(instr: &LlvmInstr, mgr: &mut TempManager) -> Result<InstrNode> {
		Ok(InstrNode {
			in_deg: 0,
			succ: Vec::new(),
			instr: to_riscv(instr, mgr)?,
		})
	}
}

impl InstrDag {
	pub fn new(instrs: &LlvmInstrSet, mgr: &mut TempManager) -> Result<InstrDag> {
		let mut nodes = Vec::new();
		let mut uses = HashMap::new();
		let mut defs = HashMap::new();
		let mut loads = Vec::new();
		let mut stores = Vec::new();
		for instr in instrs.iter().rev() {
			let mut succ: Vec<Node> = Vec::new();
			let node = Rc::new(RefCell::new(InstrNode::new(instr, mgr)?));
			if let Some(target) = instr.get_write() {
				succ.extend(uses.remove(&target).unwrap_or_default());
				defs.insert(target, node.clone());
			}
			for temp in instr.get_read() {
				if let Some(def) = defs.get(&temp) {
					succ.push(def.clone());
				}
				uses.entry(temp).or_insert_with(Vec::new).push(node.clone());
			}
			if instr.is_load() {
				succ.append(&mut stores);
				loads.push(node.clone());
			}
			if instr.has_sideeffect() {
				succ.append(&mut loads);
				stores.push(node.clone());
			}
			node.borrow_mut().succ = succ;
			nodes.push(node);
		}
		Ok(InstrDag { nodes })
	}
}
