use std::collections::{HashSet, VecDeque};

use crate::{context::IRPassContext, irpass::IRPass};
use llvm::cfg::CFG;

use llvm::LlvmProgram;

pub struct DeadcodeRemove {}

impl DeadcodeRemove {
	pub fn new() -> Self {
		DeadcodeRemove {}
	}
}

impl Default for DeadcodeRemove {
	fn default() -> Self {
		Self::new()
	}
}

impl IRPass for DeadcodeRemove {
	fn pass(&mut self, program: &mut LlvmProgram, _context: &mut IRPassContext) {
		for func in program.funcs.iter_mut() {
			remove_dead_code(&mut func.cfg);
		}
	}
}

fn remove_dead_code(cfg: &mut CFG) {
	let mut reachable = HashSet::new();
	let mut worklist = VecDeque::new();
	worklist.push_back(cfg.entry);
	while let Some(bb_id) = worklist.pop_front() {
		if reachable.contains(&bb_id) {
			continue;
		}
		reachable.insert(bb_id);
		let bb = cfg.basic_blocks.get(&bb_id).unwrap();
		for succ_id in &bb.succ {
			worklist.push_back(*succ_id);
		}
	}
	let mut ids = Vec::new();
	for id in cfg.basic_blocks.keys() {
		ids.push(*id);
	}
	for id in ids {
		if !reachable.contains(&id) {
			cfg.basic_blocks.remove(&id);
		}
	}
}
