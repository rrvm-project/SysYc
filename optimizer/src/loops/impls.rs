use crate::{loops::loop_unroll::loop_unroll, RrvmOptimizer};
use llvm::LlvmTempManager;
use rrvm::program::{LlvmFunc, LlvmProgram};
use utils::errors::Result;

use super::HandleLoops;

impl RrvmOptimizer for HandleLoops {
	fn new() -> Self {
		Self {}
	}
	fn apply(self, program: &mut LlvmProgram) -> Result<bool> {
		fn solve(func: &mut LlvmFunc, temp_mgr: &mut LlvmTempManager) -> bool {
			let cfg = &mut func.cfg;
			let flag: bool = false;
			cfg.compute_dominator();
			let loops = cfg.loop_analysis();
			for loop_ in loops {
				loop_unroll(func, loop_, temp_mgr);
			}
			flag
		}

		Ok(program.funcs.iter_mut().fold(false, |last, func| {
			solve(func, &mut program.temp_mgr) || last
		}))
	}
}
