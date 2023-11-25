use std::{
	collections::{HashMap, HashSet},
	fmt::Display,
};
use utils::Label;

use crate::{LlvmInstr, Temp};

use std::env;

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
	pub phi_instrs_vec: Vec<Box<dyn LlvmInstr>>,
	pub symbol2temp: HashMap<usize, Temp>,
	pub phi_instrs: HashMap<Temp, Vec<(Label, Temp)>>,
}

impl BasicBlock {
	pub fn new(
		id: usize,
		label: Label,
		instrs: Vec<Box<dyn LlvmInstr>>,
	) -> BasicBlock {
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
			symbol2temp: HashMap::new(),
			phi_instrs: HashMap::new(),
			phi_instrs_vec: Vec::new(),
		}
	}
	pub fn add(&mut self, instr: Box<dyn LlvmInstr>) {
		self.instrs.push(instr);
	}
}

impl Display for BasicBlock {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		if let Ok(v) = env::var("debugmiddle") {
			if v == "1" {
				write!(f, "Basicblock id: {} ", self.id)?;
				write!(f, "pred: {:?} ", self.pred)?;
				writeln!(f, "succ: {:?}", self.succ)?;
			}
		}

		writeln!(f, "  {}:", self.label)?;
		for instr in &self.phi_instrs_vec {
			writeln!(f, "  {}", instr)?;
		}
		for instr in &self.instrs {
			writeln!(f, "  {}", instr)?;
		}
		Ok(())
	}
}
