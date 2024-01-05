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
	pub last_use: usize,
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
			last_use: 0,
		})
	}
}

#[derive(PartialEq, Eq)]
enum LastState {
	Load,
	Store,
	Call,
}

impl InstrDag {
	pub fn new(instrs: &LlvmInstrSet, mgr: &mut TempManager) -> Result<InstrDag> {
		use LastState::*;
		let mut nodes = Vec::new();
		let mut uses = HashMap::new();
		let mut defs = HashMap::new();
		let mut loads = Vec::new();
		let mut stores = Vec::new();
		let mut last_state = Call;
		let mut last_use = HashMap::new();
		for (index, instr) in instrs.iter().enumerate().rev() {
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
				uses.entry(temp.clone()).or_insert_with(Vec::new).push(node.clone());
				last_use.entry(temp).or_insert(index);
			}
			if instr.is_load() {
				if last_state != Load {
					last_state = Load;
					loads.clear();
				}
				succ.extend(stores.clone());
				loads.push(node.clone());
			}
			if instr.is_store() {
				if last_state != Store {
					last_state = Store;
					stores.clear();
				}
				succ.extend(loads.clone());
				stores.push(node.clone());
			}
			if instr.is_call() {
				succ.extend(loads.clone());
				succ.extend(stores.clone());
				stores = vec![node.clone()];
				loads = vec![node.clone()];
				last_state = Call;
			}
			node.borrow_mut().succ = succ;
			nodes.push(node);
		}
		for (index, instr) in nodes.iter_mut().enumerate().rev() {
			instr.borrow_mut().last_use +=
				last_use.iter().filter(|x| *x.1 == index).count();
		}
		Ok(InstrDag { nodes })
	}
}
