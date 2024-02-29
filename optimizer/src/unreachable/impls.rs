use std::collections::HashSet;

use super::RemoveUnreachCode;
use crate::RrvmOptimizer;
use rrvm::program::LlvmProgram;
use utils::errors::Result;

impl RrvmOptimizer for RemoveUnreachCode {
	fn new() -> Self {
		Self {}
	}
	fn apply(self, program: &mut LlvmProgram) -> Result<bool> {
		Ok(program.funcs.iter_mut().fold(false, |last, func| {
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
			last || size != cfg.blocks.len()
		}))
	}
}
