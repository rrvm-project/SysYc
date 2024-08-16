use crate::{useless_phis::RemoveUselessPhis, *};
use alloc_hoisting::AllocHoisting;
use code_hoisting::CodeHoisting;
use dead_code::RemoveDeadCode;
use fold_constants::FoldConstants;
use function_inline::InlineFunction;
use global_analysis::GlobalAnalysis;
use global_value_numbering::GlobalValueNumbering;
use loops::HandleLoops;
use mem2reg::Mem2Reg;
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
			flag |= InlineFunction::new().apply(program, &mut metadata)?;
			flag |= AllocHoisting::new().apply(program, &mut metadata)?;
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
		RemoveUnreachCode::new().apply(program, &mut metadata)?;

		loop {
			let mut flag = false;
			flag |= RemoveDeadCode::new().apply(program, &mut metadata)?;
			flag |= GlobalAnalysis::new().apply(program, &mut metadata)?;
			flag |= RemoveUselessCode::new().apply(program, &mut metadata)?;
			flag |= RemoveUnreachCode::new().apply(program, &mut metadata)?;
			flag |= FoldConstants::new().apply(program, &mut metadata)?;
			flag |= GlobalValueNumbering::new().apply(program, &mut metadata)?;
			flag |= Mem2Reg::new().apply(program, &mut metadata)?;
			flag |= RemoveUselessPhis::new().apply(program, &mut metadata)?;
			flag |= InlineFunction::new().apply(program, &mut metadata)?;
			flag |= AllocHoisting::new().apply(program, &mut metadata)?;
			flag |= CodeHoisting::new().apply(program, &mut metadata)?;
			flag |= SolveTailRecursion::new().apply(program, &mut metadata)?;
			if !flag {
				break;
			}
		}

		let mut loop_handler = HandleLoops::new(program);
		loop_handler.loop_simplify(program, &mut metadata)?;
		loop_handler.indvar_extraction(program, &mut metadata)?;

		loop {
			let mut flag = false;
			flag |= RemoveDeadCode::new().apply(program, &mut metadata)?;
			flag |= GlobalAnalysis::new().apply(program, &mut metadata)?;
			flag |= RemoveUselessCode::new().apply(program, &mut metadata)?;
			flag |= RemoveUnreachCode::new().apply(program, &mut metadata)?;
			flag |= GlobalAnalysis::new().apply(program, &mut metadata)?;
			flag |= FoldConstants::new().apply(program, &mut metadata)?;
			flag |= GlobalValueNumbering::new().apply(program, &mut metadata)?;
			flag |= Mem2Reg::new().apply(program, &mut metadata)?;
			flag |= RemoveUselessPhis::new().apply(program, &mut metadata)?;
			flag |= InlineFunction::new().apply(program, &mut metadata)?;
			flag |= AllocHoisting::new().apply(program, &mut metadata)?;
			flag |= CodeHoisting::new().apply(program, &mut metadata)?;
			flag |= SolveTailRecursion::new().apply(program, &mut metadata)?;
			if !flag {
				break;
			}
		}

		program.analysis();
		Ok(())
	}
}
