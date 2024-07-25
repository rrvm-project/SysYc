use crate::{
	loops::{loop_simplify::simplify_one_loop, loop_unroll::loop_unroll},
	RrvmOptimizer,
};
use llvm::LlvmTempManager;
use rrvm::program::{LlvmFunc, LlvmProgram};
use utils::errors::Result;

use super::HandleLoops;

impl RrvmOptimizer for HandleLoops {
	fn new() -> Self {
		Self {}
	}
	fn apply(self, program: &mut LlvmProgram) -> Result<bool> {
		fn solve(
			func: &mut LlvmFunc,
			temp_mgr: &mut LlvmTempManager,
			loop_unroll_time: &mut usize,
		) -> bool {
			let flag: bool = false;
			func.cfg.compute_dominator();
			let loops = func.cfg.loop_analysis();
			for loop_ in loops.iter() {
				simplify_one_loop(func, loop_.clone(), temp_mgr);
			}
			func.cfg.compute_dominator();
			for loop_ in loops {
				loop_unroll(func, loop_, temp_mgr, loop_unroll_time);
			}
			flag
		}

		let mut loop_unroll_time = 0;

		let result = program.funcs.iter_mut().fold(false, |last, func| {
			solve(func, &mut program.temp_mgr, &mut loop_unroll_time) || last
		});

		if loop_unroll_time > 0 {
			println!("loop unroll time: {}", loop_unroll_time);
		}

		Ok(result)
	}
}
