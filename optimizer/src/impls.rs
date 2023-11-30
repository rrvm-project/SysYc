use rrvm::program::LlvmProgram;

use crate::{BasicOptimizer, RrvmOptimizer};

impl RrvmOptimizer for BasicOptimizer {
	fn new() -> Self {
		Self {}
	}
	fn apply(self, _program: LlvmProgram) -> LlvmProgram {
		todo!()
	}
}
