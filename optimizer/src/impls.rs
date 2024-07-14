use crate::{useless_phis::RemoveUselessPhis, *};
use dead_code::RemoveDeadCode;
use fold_constants::FoldConstants;
use function_inline::InlineFunction;
use global_value_numbering::GlobalValueNumbering;
use strength_reduce::StrengthReduce;
use tail_recursion::SolveTailRecursion;
use unreachable::RemoveUnreachCode;
use useless_code::RemoveUselessCode;
use useless_phis::RemoveUselessPhis;

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
			// eprintln!("{}", program);
			flag |= RemoveUselessCode::new().apply(program, &mut metadata)?;
			flag |= RemoveUnreachCode::new().apply(program, &mut metadata)?;
			flag |= FoldConstants::new().apply(program, &mut metadata)?;
			// eprintln!("=================================\n{}", program);
			flag |= GlobalValueNumbering::new().apply(program, &mut metadata)?;
			flag |= RemoveUselessPhis::new().apply(program, &mut metadata)?;
			// flag |= MemoryInstrElimination::new().apply(program, &mut metadata)?;
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
			flag |= RemoveUselessPhis::new().apply(program, &mut metadata)?;
			// flag |= MemoryInstrElimination::new().apply(program, &mut metadata)?;
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
