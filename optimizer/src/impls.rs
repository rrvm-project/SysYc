use rrvm::program::LlvmProgram;
use utils::errors::Result;

use crate::{dead_code::RemoveDeadCode, BasicOptimizer, RrvmOptimizer};

impl RrvmOptimizer for BasicOptimizer {
	fn new() -> Self {
		Self {}
	}
	fn apply(self, program: &mut LlvmProgram) -> Result<()> {
		RemoveDeadCode::new().apply(program)
	}
}
