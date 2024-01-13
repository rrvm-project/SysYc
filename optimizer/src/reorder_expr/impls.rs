use super::ReorderExpr;
use crate::RrvmOptimizer;
use rrvm::{program::LlvmProgram, LlvmNode};
use utils::errors::Result;

impl RrvmOptimizer for ReorderExpr {
	fn new() -> Self {
		Self {}
	}
	fn apply(self, program: &mut LlvmProgram) -> Result<bool> {
		fn solve(block: &LlvmNode) {
			let _block = &mut block.borrow_mut();
			// for
		}
		program.analysis();
		program
			.funcs
			.iter()
			.for_each(|func| func.cfg.blocks.iter().for_each(solve));
		Ok(false)
	}
}
