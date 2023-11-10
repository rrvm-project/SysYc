use std::collections::HashSet;

use instruction::InstrSet;
use llvm::{label::Label, temp::Temp};

pub struct BasicBlock {
	pub id: usize,
	pub pred: Vec<usize>,
	pub succ: Vec<usize>,
	pub label: Option<Label>,
	pub defs: HashSet<Temp>,
	pub uses: HashSet<Temp>,
	pub live_in: HashSet<Temp>,
	pub live_out: HashSet<Temp>,
	pub instrs: InstrSet,
}

impl BasicBlock {
	pub fn new(id: usize, label: Option<Label>, instrs: InstrSet) -> BasicBlock {
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
}
