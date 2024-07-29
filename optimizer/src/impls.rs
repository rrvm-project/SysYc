use crate::{
	strength_reduce::StrengthReduce, useless_phis::RemoveUselessPhis, *,
};
use dead_code::RemoveDeadCode;
use fold_constants::FoldConstants;
use function_inline::InlineFunction;
use fuyuki_vn::{FuyukiLocalValueNumber, GLobalValueNumber};
use localize_variable::LocalizeVariable;
use loop_parallel::LoopParallel;
use loops::HandleLoops;
use simplify_compare::SimplifyCompare;
use tail_recursion::SolveTailRecursion;
use unreachable::RemoveUnreachCode;
use useless_code::RemoveUselessCode;
use zero_init::ZeroInit;

use self::pure_check::PureCheck;

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
			// eprintln!("{}", program);
			flag |= RemoveDeadCode::new().apply(program)?;
			flag |= RemoveUnreachCode::new().apply(program)?;
			flag |= RemoveUselessCode::new().apply(program)?;
			flag |= FoldConstants::new().apply(program)?;
			flag |= PureCheck::new().apply(program)?;
			// // flag |= FuyukiLocalValueNumber::new().apply(program)?;
			flag |= GLobalValueNumber::new().apply(program)?;
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
		// 需在表达式重排前进行，否则，运算指令分布在不同的基本块中， LER做不了任何事情
		RemoveDeadCode::new().apply(program)?;
		RemoveUselessCode::new().apply(program)?;
		RemoveUnreachCode::new().apply(program)?;
		RemoveUselessCode::new().apply(program)?;
		ZeroInit::new().apply(program)?;
		// eprintln!("{}", &program);
		loop {
			let mut flag = false;
			flag |= RemoveDeadCode::new().apply(program)?;
			flag |= RemoveUnreachCode::new().apply(program)?;
			flag |= RemoveUselessCode::new().apply(program)?;
			flag |= PureCheck::new().apply(program)?;
			flag |= LocalizeVariable::new().apply(program)?;
			flag |= SimplifyCompare::new().apply(program)?;
			
			// eprintln!("{}", &program);
			flag |= FoldConstants::new().apply(program)?;
			flag |= FuyukiLocalValueNumber::new().apply(program)?;
			flag |= GLobalValueNumber::new().apply(program)?;
			flag |= RemoveUselessPhis::new().apply(program)?;
			flag |= InlineFunction::new().apply(program)?;
			flag |= SolveTailRecursion::new().apply(program)?;
			if !flag {
				break;
			}
		}

		StrengthReduce::new().apply(program)?;
		LoopParallel::new().apply(program)?;
		eprintln!("{}", &program);
		
		loop {
			let mut flag = false;
			flag |= RemoveDeadCode::new().apply(program)?;
			flag |= RemoveUnreachCode::new().apply(program)?;
			flag |= RemoveUselessCode::new().apply(program)?;
			flag |= FoldConstants::new().apply(program)?;
			flag |= LocalizeVariable::new().apply(program)?;
			flag |= SimplifyCompare::new().apply(program)?;
			// eprintln!("{}", &program);
			flag |= FuyukiLocalValueNumber::new().apply(program)?;
			flag |= RemoveUselessPhis::new().apply(program)?;
			flag |= InlineFunction::new().apply(program)?;
			flag |= SolveTailRecursion::new().apply(program)?;
			flag |= HandleLoops::new().apply(program)?;
			if !flag {
				break;
			}
		}
		program.analysis();
		Ok(())
	}
}
