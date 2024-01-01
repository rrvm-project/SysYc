use rrvm::program::LlvmProgram;
use utils::errors::Result;

use crate::{strength_reduce::StrengthReduce, RrvmOptimizer, *};
use dead_code::RemoveDeadCode;
use local_expression_rearrangement::LocalExpressionRearrangement;
use unreachable::RemoveUnreachCode;
use useless_code::RemoveUselessCode;

impl Optimizer0 {
	pub fn new() -> Self {
		Self::default()
	}
	pub fn apply(self, program: &mut LlvmProgram) -> Result<()> {
		loop {
			let mut flag = false;
			flag |= RemoveUnreachCode::new().apply(program)?;
			if !flag {
				break;
			}
		}
		program.analysis();
		Ok(())
	}
}

impl Optimizer1 {
	pub fn new() -> Self {
		Self::default()
	}
	pub fn apply(mut self, program: &mut LlvmProgram) -> Result<()> {
		LocalExpressionRearrangement::new().apply(program)?;
		RemoveUselessCode::new().apply(program)?;
		loop {
			let mut flag = false;
			flag |= RemoveDeadCode::new().apply(program)?;
			flag |= RemoveUselessCode::new().apply(program)?;
			flag |= RemoveUnreachCode::new().apply(program)?;
			let (strength_reduce_flag, strength_reduce_total_new_temp) =
				StrengthReduce::new_with_total_new_temp(
					self.strength_reduce_total_new_temp,
				)
				.apply_strength_reduce(program)?;
			flag |= strength_reduce_flag;
			self.strength_reduce_total_new_temp = strength_reduce_total_new_temp;
			if !flag {
				break;
			}
		}
		program.analysis();
		Ok(())
	}
}

impl Optimizer2 {
	pub fn new() -> Self {
		Self::default()
	}
	pub fn apply(self, _program: &mut LlvmProgram) -> Result<bool> {
		todo!()
	}
}
