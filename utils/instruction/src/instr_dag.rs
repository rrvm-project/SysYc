use std::{cell::RefCell, collections::HashMap, rc::Rc};

use llvm::llvminstr::LlvmInstr;
use utils::errors::Result;

use crate::{temp::TempManager, transformer::to_riscv, InstrSet};

type Node = Rc<RefCell<InstrNode>>;

pub struct InstrNode {
	pub in_deg: usize,
	pub instr: InstrSet,
	pub succ: Vec<Node>,
}

pub struct InstrDag {
	pub nodes: Vec<Node>,
}

impl InstrNode {
	pub fn new(instr: Box<dyn LlvmInstr>, succ: Vec<Node>) -> InstrNode {
		InstrNode {
			in_deg: instr.get_read().len(),
			instr: InstrSet::LlvmInstrSet(vec![instr]),
			succ,
		}
	}
	pub fn convert(&mut self, mgr: &mut TempManager) -> Result<()> {
		to_riscv(&mut self.instr, mgr)?;
		Ok(())
	}
}

impl InstrDag {
	pub fn new(block: InstrSet) -> InstrDag {
		let instrs = match block {
			InstrSet::LlvmInstrSet(v) => v,
			_ => unreachable!("？你都不是 llvm 指令还要我干什么"),
		};
		let mut nodes = Vec::new();
		let mut edge = HashMap::new();
		for instr in instrs.into_iter().rev() {
			let prev = instr.get_read();
			let node = if let Some(target) = instr.get_write() {
				InstrNode::new(instr, edge.remove(&target).unwrap_or(Vec::new()))
			} else {
				InstrNode::new(instr, Vec::new())
			};
			let node = Rc::new(RefCell::new(node));
			for label in prev {
				edge.entry(label).or_insert_with(Vec::new).push(Rc::clone(&node));
			}
			nodes.push(node);
		}
		InstrDag { nodes }
	}
	pub fn convert(&mut self) -> Result<()> {
		let mut mgr = TempManager::new();
		for node in self.nodes.iter_mut() {
			node.borrow_mut().convert(&mut mgr)?;
		}
		Ok(())
	}
}
