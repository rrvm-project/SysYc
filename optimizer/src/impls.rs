use crate::{useless_phis::RemoveUselessPhis, *};
use dead_code::RemoveDeadCode;
use fold_constants::FoldConstants;
use function_inline::InlineFunction;
use global_value_numbering::GlobalValueNumbering;
use mem2reg::Mem2Reg;
use strength_reduce::StrengthReduce;
use tail_recursion::SolveTailRecursion;
use unreachable::RemoveUnreachCode;
use useless_code::RemoveUselessCode;

impl Optimizer0 {
	pub fn new() -> Self {
		Self::default()
	}
	pub fn apply(self, program: &mut LlvmProgram) -> Result<()> {
		let mut metadata = MetaData::new();
		loop {
			let mut flag = false;
			flag |= RemoveUnreachCode::new().apply(program, &mut metadata)?;
			flag |= RemoveDeadCode::new().apply(program, &mut metadata)?;
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
		let mut metadata = MetaData::new();
		loop {
			let mut flag = false;
			flag |= RemoveDeadCode::new().apply(program, &mut metadata)?;
			flag |= RemoveUnreachCode::new().apply(program, &mut metadata)?;
			flag |= RemoveUselessCode::new().apply(program, &mut metadata)?;
			flag |= FoldConstants::new().apply(program, &mut metadata)?;
			flag |= RemoveUselessPhis::new().apply(program, &mut metadata)?;
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
		let mut metadata = MetaData::new();

		loop {
			let mut flag = false;
			flag |= RemoveDeadCode::new().apply(program, &mut metadata)?;
			flag |= RemoveUselessCode::new().apply(program, &mut metadata)?;
			flag |= RemoveUnreachCode::new().apply(program, &mut metadata)?;
			flag |= FoldConstants::new().apply(program, &mut metadata)?;
			flag |= GlobalValueNumbering::new().apply(program, &mut metadata)?;
			flag |= Mem2Reg::new().apply(program, &mut metadata)?;
			flag |= RemoveUselessPhis::new().apply(program, &mut metadata)?;
			flag |= InlineFunction::new().apply(program, &mut metadata)?;
			flag |= SolveTailRecursion::new().apply(program, &mut metadata)?;
			if !flag {
				break;
			}
		}

		StrengthReduce::new().apply(program, &mut metadata)?;

		loop {
			let mut flag = false;
			flag |= RemoveDeadCode::new().apply(program, &mut metadata)?;
			flag |= RemoveUselessCode::new().apply(program, &mut metadata)?;
			flag |= RemoveUnreachCode::new().apply(program, &mut metadata)?;
			flag |= FoldConstants::new().apply(program, &mut metadata)?;
			flag |= GlobalValueNumbering::new().apply(program, &mut metadata)?;
			flag |= Mem2Reg::new().apply(program, &mut metadata)?;
			flag |= RemoveUselessPhis::new().apply(program, &mut metadata)?;
			flag |= InlineFunction::new().apply(program, &mut metadata)?;
			flag |= SolveTailRecursion::new().apply(program, &mut metadata)?;
			if !flag {
				break;
			}
		}
		program.analysis();
		Ok(())
	}
}
