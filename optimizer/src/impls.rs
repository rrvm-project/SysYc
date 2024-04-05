use crate::{
	strength_reduce::StrengthReduce, useless_phis::RemoveUselessPhis, *,
};
use dead_code::RemoveDeadCode;
use fold_constants::FoldConstants;
use function_inline::InlineFunction;
use fuyuki_vn::FuyukiLocalValueNumber;
use tail_recursion::SolveTailRecursion;
use unreachable::RemoveUnreachCode;
use useless_code::RemoveUselessCode;

use self::loops::HandleLoops;
use self::pure_check::PureCheck;
type FuncPtrOfOptPass = Box<dyn Fn() -> Box<dyn RrvmOptimizer>>;

impl Optimizer0 {
	pub fn new() -> Self {
		Self::default()
	}
	pub fn apply(self, program: &mut LlvmProgram) -> Result<()> {
		let vec0: Vec<(String, FuncPtrOfOptPass)> = vec![
			(
				"RemoveUnreachCode".to_string(),
				Box::new(|| Box::new(RemoveUnreachCode::new())),
			),
			(
				"RemoveDeadCode".to_string(),
				Box::new(|| Box::new(RemoveDeadCode::new())),
			),
		];
		loop {
			let mut flag = false;
			for item in
				vec0.iter().filter(|item| !O0_IGNORE.lock().unwrap().contains(&item.0))
			{
				flag |= item.1().apply(program)?;
			}
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
		let vec0: Vec<(String, FuncPtrOfOptPass)> = vec![
			(
				"RemoveDeadCode".to_string(),
				Box::new(|| Box::new(RemoveDeadCode::new())),
			),
			(
				"RemoveUnreachCode".to_string(),
				Box::new(|| Box::new(RemoveUnreachCode::new())),
			),
			(
				"RemoveUselessCode".to_string(),
				Box::new(|| Box::new(RemoveUselessCode::new())),
			),
			(
				"FoldConstants".to_string(),
				Box::new(|| Box::new(FoldConstants::new())),
			),
			(
				"FuyukiLocalValueNumber".to_string(),
				Box::new(|| Box::new(FuyukiLocalValueNumber::new())),
			),
			(
				"RemoveUselessPhis".to_string(),
				Box::new(|| Box::new(RemoveUselessPhis::new())),
			),
		];

		loop {
			let mut flag = false;
			for item in
				vec0.iter().filter(|item| !O1_IGNORE.lock().unwrap().contains(&item.0))
			{
				flag |= item.1().apply(program)?;
			}
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
		let vec0: Vec<(String, FuncPtrOfOptPass)> = vec![
			(
				"RemoveDeadCode".to_string(),
				Box::new(|| Box::new(RemoveDeadCode::new())),
			),
			(
				"RemoveUselessCode".to_string(),
				Box::new(|| Box::new(RemoveUselessCode::new())),
			),
			(
				"RemoveUnreachCode".to_string(),
				Box::new(|| Box::new(RemoveUnreachCode::new())),
			),
			(
				"RemoveUselessCode".to_string(),
				Box::new(|| Box::new(RemoveUselessCode::new())),
			),
		];

		let vec1: Vec<(String, FuncPtrOfOptPass)> = vec![
			(
				"RemoveDeadCode".to_string(),
				Box::new(|| Box::new(RemoveDeadCode::new())),
			),
			(
				"RemoveUnreachCode".to_string(),
				Box::new(|| Box::new(RemoveUnreachCode::new())),
			),
			(
				"RemoveUselessCode".to_string(),
				Box::new(|| Box::new(RemoveUselessCode::new())),
			),
			(
				"PureCheck".to_string(),
				Box::new(|| Box::new(PureCheck::new())),
			),
			(
				"FoldConstants".to_string(),
				Box::new(|| Box::new(FoldConstants::new())),
			),
			(
				"FuyukiLocalValueNumber".to_string(),
				Box::new(|| Box::new(FuyukiLocalValueNumber::new())),
			),
			(
				"RemoveUselessPhis".to_string(),
				Box::new(|| Box::new(RemoveUselessPhis::new())),
			),
			(
				"InlineFunction".to_string(),
				Box::new(|| Box::new(InlineFunction::new())),
			),
			(
				"SolveTailRecursion".to_string(),
				Box::new(|| Box::new(SolveTailRecursion::new())),
			),
		];
		// 需在表达式重排前进行，否则，运算指令分布在不同的基本块中， LER做不了任何事情

		for item in
			vec0.iter().filter(|item| !O2_IGNORE.lock().unwrap().contains(&item.0))
		{
			item.1().apply(program)?;
		}

		loop {
			let mut flag = false;
			for item in
				vec1.iter().filter(|item| !O2_IGNORE.lock().unwrap().contains(&item.0))
			{
				flag |= item.1().apply(program)?;
			}

			if !flag {
				break;
			}
		}

		let vec2: Vec<(String, FuncPtrOfOptPass)> = vec![
			(
				"StrengthReduce".to_string(),
				Box::new(|| Box::new(StrengthReduce::new())),
			),
			(
				"HandleLoops".to_string(),
				Box::new(|| Box::new(HandleLoops::new())),
			),
		];

		for item in
			vec2.iter().filter(|item| !O2_IGNORE.lock().unwrap().contains(&item.0))
		{
			item.1().apply(program)?;
		}

		loop {
			// break;
			let mut flag = false;

			for item in
				vec1.iter().filter(|item| !O2_IGNORE.lock().unwrap().contains(&item.0))
			{
				flag |= item.1().apply(program)?;
			}

			if !flag {
				break;
			}
		}
		program.analysis();
		Ok(())
	}
}
