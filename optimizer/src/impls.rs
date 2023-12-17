use rrvm::program::LlvmProgram;
use utils::errors::Result;

use crate::{RrvmOptimizer, *};

impl RrvmOptimizer for Optimizer0 {
	fn new() -> Self {
		Self {}
	}
	fn apply(self, program: &mut LlvmProgram) -> Result<()> {
		RemoveUnreachCode::new().apply(program)?;
		program.analysis();
		Ok(())
	}
}

impl RrvmOptimizer for Optimizer1 {
	fn new() -> Self {
		Self {}
	}
	fn apply(self, program: &mut LlvmProgram) -> Result<()> {
		RemoveDeadCode::new().apply(program)?;
		RemoveUnreachCode::new().apply(program)?;
		program.analysis();
		Ok(())
	}
}

impl RrvmOptimizer for Optimizer2 {
	fn new() -> Self {
		Self {}
	}
	fn apply(self, _program: &mut LlvmProgram) -> Result<()> {
		todo!()
	}
}
