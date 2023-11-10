use std::collections::HashSet;

use llvm::{label::Label, llvminstr::LlvmInstr, temp::Temp};

pub struct BasicBlock {
	pub id: usize,
	pub pred: Vec<usize>,
	pub succ: Vec<usize>,
	pub label: Option<Label>,
	pub defs: HashSet<Temp>,
	pub uses: HashSet<Temp>,
	pub live_in: HashSet<Temp>,
	pub live_out: HashSet<Temp>,
	pub instrs: Vec<Box<dyn LlvmInstr>>,
}
