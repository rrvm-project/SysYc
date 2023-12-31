use rrvm::{program::LlvmProgram, LlvmCFG};
use utils::Result;

use crate::RrvmOptimizer;

use super::StrengthReduce;

impl RrvmOptimizer for StrengthReduce {
	fn new() -> Self {
		Self {
			total_new_temp: 0,
		}
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

impl StrengthReduce {
	fn new_with_total_new_temp(total_new_temp: u32) -> Self {
		Self { total_new_temp }
	}
}
