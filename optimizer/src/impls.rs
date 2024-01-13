use rrvm::program::LlvmProgram;
use utils::errors::Result;

use crate::{
	strength_reduce::StrengthReduce, useless_phis::RemoveUselessPhis,
	RrvmOptimizer, *,
};
use dead_code::RemoveDeadCode;
use fold_constants::FoldConstants;
use function_inline::InlineFunction;
use fuyuki_vn::FuyukiLocalValueNumber;
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
			flag |= FuyukiLocalValueNumber::new().apply(program)?;
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
	pub fn apply(mut self, program: &mut LlvmProgram) -> Result<()> {
		// 需在表达式重排前进行，否则，运算指令分布在不同的基本块中， LER做不了任何事情
		RemoveDeadCode::new().apply(program)?;
		RemoveUselessCode::new().apply(program)?;
		RemoveUnreachCode::new().apply(program)?;

		LocalExpressionRearrangement::new().apply(program)?;
		RemoveUselessCode::new().apply(program)?;
		loop {
			let mut flag = false;
			flag |= RemoveDeadCode::new().apply(program)?;
			flag |= RemoveUnreachCode::new().apply(program)?;
			flag |= RemoveUselessCode::new().apply(program)?;
			flag |= FoldConstants::new().apply(program)?;
			flag |= FuyukiLocalValueNumber::new().apply(program)?;
			flag |= RemoveUselessPhis::new().apply(program)?;
			flag |= InlineFunction::new().apply(program)?;
			flag |= SolveTailRecursion::new().apply(program)?;
			// 	}
			// }

			if !flag {
				break;
			}
		}

		let (_, strength_reduce_total_new_temp) =
			StrengthReduce::new_with_total_new_temp(
				self.strength_reduce_total_new_temp,
			)
			.apply_strength_reduce(program)?;
		self.strength_reduce_total_new_temp = strength_reduce_total_new_temp;

		loop {
			let mut flag = false;
			flag |= RemoveDeadCode::new().apply(program)?;
			flag |= RemoveUselessCode::new().apply(program)?;
			flag |= RemoveUnreachCode::new().apply(program)?;
			if !flag {
				break;
			}
		}

		program.analysis();
		Ok(())
	}
}
