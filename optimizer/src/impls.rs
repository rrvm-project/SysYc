#![allow(unused)]
use crate::{
	strength_reduce::StrengthReduce, useless_phis::RemoveUselessPhis, *,
};
use dead_code::RemoveDeadCode;
use fold_constants::FoldConstants;
use function_inline::InlineFunction;
use global_value_numbering::GlobalValueNumbering;
use tail_recursion::SolveTailRecursion;
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
			flag |= RemoveDeadCode::new().apply(program)?;
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
	pub fn apply(self, program: &mut LlvmProgram) -> Result<()> {
		loop {
			let mut flag = false;
			flag |= RemoveDeadCode::new().apply(program)?;
			flag |= RemoveUnreachCode::new().apply(program)?;
			flag |= RemoveUselessCode::new().apply(program)?;
			flag |= FoldConstants::new().apply(program)?;
			flag |= RemoveUselessPhis::new().apply(program)?;
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
	pub fn apply(self, program: &mut LlvmProgram) -> Result<()> {
		loop {
			let mut flag = false;
			flag |= RemoveDeadCode::new().apply(program)?;
			flag |= RemoveUselessCode::new().apply(program)?;
			flag |= RemoveUnreachCode::new().apply(program)?;
			flag |= FoldConstants::new().apply(program)?;
			flag |= GlobalValueNumbering::new().apply(program)?;
			flag |= RemoveUselessPhis::new().apply(program)?;
			flag |= InlineFunction::new().apply(program)?;
			flag |= SolveTailRecursion::new().apply(program)?;
			if !flag {
				break;
			}
		}

		StrengthReduce::new().apply(program)?;

		loop {
			let mut flag = false;
			flag |= RemoveDeadCode::new().apply(program)?;
			flag |= RemoveUselessCode::new().apply(program)?;
			flag |= RemoveUnreachCode::new().apply(program)?;
			flag |= FoldConstants::new().apply(program)?;
			flag |= GlobalValueNumbering::new().apply(program)?;
			flag |= RemoveUselessPhis::new().apply(program)?;
			flag |= InlineFunction::new().apply(program)?;
			flag |= SolveTailRecursion::new().apply(program)?;
			if !flag {
				break;
			}
		}
		program.analysis();
		Ok(())
	}
}
