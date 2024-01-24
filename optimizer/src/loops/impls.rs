use crate::{loops::loop_unroll::loop_unroll, RrvmOptimizer};
use llvm::Value;
use rrvm::{program::LlvmProgram, LlvmCFG};
use utils::errors::Result;

use super::HandleLoops;

impl RrvmOptimizer for HandleLoops {
	fn new() -> Self {
		Self {}
	}
	fn apply(self, program: &mut LlvmProgram) -> Result<bool> {
		fn solve(cfg: &mut LlvmCFG, func_params: &[Value]) -> bool {
			let flag: bool = false;
			cfg.compute_dominator();
			let loops = cfg.loop_analysis();
			for loop_ in loops {
				loop_unroll(cfg, loop_, func_params);
			}
			flag
		}

		Ok(program.funcs.iter_mut().fold(false, |last, func| {
			solve(&mut func.cfg, &func.params) || last
		}))
	}
}
