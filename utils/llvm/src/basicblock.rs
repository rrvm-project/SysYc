use std::collections::HashSet;
use utils::Label;

use crate::{LlvmInstr, Temp};

pub struct BasicBlock {
	pub id: usize,
	pub pred: Vec<usize>,
	pub succ: Vec<usize>,
	pub label: Label,
	pub defs: HashSet<Temp>,
	pub uses: HashSet<Temp>,
	pub live_in: HashSet<Temp>,
	pub live_out: HashSet<Temp>,
	pub instrs: Vec<Box<dyn LlvmInstr>>,
}

impl BasicBlock {
	pub fn new(id: usize, label: Label, instrs: Vec<Box<dyn LlvmInstr>>) -> BasicBlock {
		BasicBlock {
			id,
			label,
			instrs,
			pred: Vec::new(),
			succ: Vec::new(),
			defs: HashSet::new(),
			uses: HashSet::new(),
			live_in: HashSet::new(),
			live_out: HashSet::new(),
		}
	}
	pub fn add(&mut self, instr: Box<dyn LlvmInstr>) {
		self.instrs.push(instr);
	}
}
