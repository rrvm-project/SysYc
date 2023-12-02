use llvm::LlvmInstr;
use rrvm::cfg::Node;

use crate::symbol_table::Table;

pub struct LoopState {
	pub size: usize,
	pub entry: Vec<(Node<LlvmInstr>, Table)>,
	pub exit: Vec<(Node<LlvmInstr>, Table)>,
}

impl LoopState {
	pub fn new(size: usize) -> Self {
		Self {
			size,
			entry: Vec::new(),
			exit: Vec::new(),
		}
	}
	pub fn push_entry(&mut self, node: Node<LlvmInstr>, table: Table) {
		self.entry.push((node, table))
	}
	pub fn push_exit(&mut self, node: Node<LlvmInstr>, table: Table) {
		self.exit.push((node, table))
	}
}
