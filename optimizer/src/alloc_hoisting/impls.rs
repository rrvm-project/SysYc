use rrvm::program::{LlvmFunc, LlvmProgram};

use crate::{metadata::MetaData, RrvmOptimizer};

use super::AllocHoisting;

use utils::Result;

impl RrvmOptimizer for AllocHoisting {
	fn new() -> Self {
		Self {}
	}

	fn apply(
		self,
		program: &mut LlvmProgram,
		_metadata: &mut MetaData,
	) -> Result<bool> {
		fn solve(func: &LlvmFunc) -> bool {
			let mut allocs = Vec::new();
			for block in func.cfg.blocks.iter().skip(1) {
				block.borrow_mut().instrs.retain(|instr| {
					instr.get_alloc().is_none() || {
						allocs.push(instr.clone());
						false
					}
				})
			}
			if allocs.is_empty() {
				return false;
			}
			func.cfg.get_entry().borrow_mut().instrs.extend(allocs);
			true
		}

		Ok(program.funcs.iter().fold(false, |last, func| solve(func) || last))
	}
}
