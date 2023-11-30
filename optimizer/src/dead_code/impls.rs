use super::RemoveDeadCode;
use crate::RrvmOptimizer;
use rrvm::program::LlvmProgram;
use utils::errors::Result;

impl RrvmOptimizer for RemoveDeadCode {
	fn new() -> Self {
		Self {}
	}
	fn apply(self, _program: &mut LlvmProgram) -> Result<()> {
		todo!()
	}
}
