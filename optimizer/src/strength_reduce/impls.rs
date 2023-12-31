use rrvm::{program::LlvmProgram, LlvmCFG};
use utils::Result;

use crate::RrvmOptimizer;

use super::StrengthReduce;

impl RrvmOptimizer for StrengthReduce {
	fn new() -> Self {
		Self {}
	}
	fn apply(self, program: &mut LlvmProgram) -> Result<bool> {
		fn solve(cfg: &mut LlvmCFG) -> bool {
			let mut flag = false;
			flag;
			todo!()
		}

		Ok(
			program
				.funcs
				.iter_mut()
				.fold(false, |last, func| solve(&mut func.cfg) || last),
		)
	}
}
