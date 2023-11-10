use llvm::{label::Label, llvminstr::LlvmInstr, temp::Temp};
use std::collections::HashSet;
#[derive(Debug, Clone)]
pub enum BlockType {
	Continuous,
	EndByCondbr,
	EndByBr,
	EndByRet,
}
pub struct BasicBlock {
	pub id: i32,
	pub pred: Vec<i32>,
	pub succ: Vec<i32>,
	pub label: Option<Label>,
	pub defs: HashSet<Temp>,
	pub uses: HashSet<Temp>,
	pub live_in: HashSet<Temp>,
	pub live_out: HashSet<Temp>,
	pub instrs: Vec<Box<dyn LlvmInstr>>,
}

impl BasicBlock {
	// pub fn new(
	// 	label: Option<Label>,
	// 	id: i32,
	// 	start: i32,
	// 	end: i32,
	// ) -> BasicBlock {
	// 	BasicBlock {
	// 		id,
	// 		pred: Vec::new(),
	// 		succ: Vec::new(),
	// 		label,
	// 		range: (start, end),
	// 		defs: BTreeSet::new(),
	// 		liveuse: BTreeSet::new(),
	// 		livein: BTreeSet::new(),
	// 		liveout: BTreeSet::new(),
	// 	}
	// }
}
