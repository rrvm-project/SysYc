use rrvm::{program::LlvmProgram, LlvmCFG};
use utils::errors::Result;

use crate::{loops::LoopOptimizer, RrvmOptimizer};

use super::HandleLoops;

impl RrvmOptimizer for HandleLoops {
	fn new() -> Self {
		Self {}
	}
	fn apply(self, program: &mut LlvmProgram) -> Result<bool> {
		fn solve(cfg: &mut LlvmCFG) -> bool {
			let mut flag: bool = false;
			let mut opter = LoopOptimizer::new();
			let root_loop = cfg.loop_analysis();
			flag |= opter.apply(root_loop.clone(), cfg);
			flag
		}

		Ok(
			program
				.funcs
				.iter_mut()
				.fold(false, |last, func| solve(&mut func.cfg) || last),
		)
	}
}
