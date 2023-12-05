use rrvm::LlvmNode;

use crate::symbol_table::Table;

pub struct LoopState {
	pub size: usize,
	pub entry: Vec<(LlvmNode, Table)>,
	pub exit: Vec<(LlvmNode, Table)>,
}

impl LoopState {
	pub fn new(size: usize) -> Self {
		Self {
			size,
			entry: Vec::new(),
			exit: Vec::new(),
		}
	}
	pub fn push_entry(&mut self, node: LlvmNode, table: Table) {
		self.entry.push((node, table))
	}
	pub fn push_exit(&mut self, node: LlvmNode, table: Table) {
		self.exit.push((node, table))
	}
}
