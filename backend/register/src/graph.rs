use std::collections::HashMap;

use instruction::temp::Temp;
use rrvm::RiscvCFG;

pub struct InterferenceGraph {
	pub total: usize,
	pub color_cnt: usize,
	pub edge: Vec<Vec<usize>>,
	pub spill_node: Option<Temp>,
	pub table: HashMap<Temp, usize>,
	pub color: HashMap<Temp, usize>,
}

impl InterferenceGraph {
	pub fn new(_cfg: &RiscvCFG) -> Self {
		todo!()
	}
}
