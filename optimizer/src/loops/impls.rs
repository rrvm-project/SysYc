use llvm::LlvmTempManager;
use rrvm::program::{LlvmFunc, LlvmProgram};
use utils::errors::Result;

use crate::{
	loops::{loop_optimizer::LoopOptimizer, utils::print_all_loops},
	RrvmOptimizer,
};

use super::HandleLoops;

impl RrvmOptimizer for HandleLoops {
	fn new() -> Self {
		Self {}
	}
	fn apply(self, program: &mut LlvmProgram) -> Result<bool> {
		fn solve(func: &mut LlvmFunc, temp_mgr: &mut LlvmTempManager) -> bool {
			let mut flag: bool = false;
			let mut opter = LoopOptimizer::new();
			let root_loop = func.cfg.loop_analysis(&mut opter.loop_map);
			print_all_loops(root_loop.clone());
			flag |= opter.apply(root_loop.clone(), func, temp_mgr);
			flag
		}

		Ok(program.funcs.iter_mut().fold(false, |last, func| {
			solve(func, &mut program.temp_mgr) || last
		}))
	}
}
