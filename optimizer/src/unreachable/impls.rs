use std::collections::HashSet;

use super::RemoveUnreachCode;
use crate::{metadata::MetaData, RrvmOptimizer};
use rrvm::program::LlvmProgram;
use utils::errors::Result;

impl RrvmOptimizer for RemoveUnreachCode {
	fn new() -> Self {
		Self {}
	}
	fn apply(
		self,
		program: &mut LlvmProgram,
		_metadata: &mut MetaData,
	) -> Result<bool> {
		let flag = program.funcs.iter_mut().fold(false, |last, func| {
			let size = func.cfg.size();
			let mut visited = HashSet::new();
			let cfg = &mut func.cfg;
			let mut stack = vec![cfg.get_entry()];
			while let Some(u) = stack.pop() {
				let id = u.borrow().id;
				visited.insert(id);
				// dfs
				for v in u.borrow().succ.iter() {
					if !visited.contains(&v.borrow().id) {
						stack.push(v.clone())
					}
				}
			}
			cfg.blocks.retain(|v| {
				visited.contains(&v.borrow().id) || {
					v.borrow_mut().clear();
					false
				}
			});
			for block in cfg.blocks.iter() {
				block.borrow_mut().prev.retain(|v| visited.contains(&v.borrow().id));
			}
			last || size != cfg.blocks.len()
		});
		let mut used_func = HashSet::new();
		used_func.insert("main".into());
		for func in program.funcs.iter() {
			for block in func.cfg.blocks.iter() {
				for instr in block.borrow().instrs.iter() {
					if instr.is_call() {
						used_func.insert(instr.get_label().name);
					}
				}
			}
		}
		program.funcs.retain(|func| used_func.contains(&func.name));
		Ok(flag)
	}
}
