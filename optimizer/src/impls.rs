use rrvm::program::LlvmProgram;
use utils::errors::Result;

use crate::{RrvmOptimizer, *};
use dead_code::RemoveDeadCode;
use fuyuki_vn::FuyukiLocalValueNumber;
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
	pub fn apply(self, program: &mut LlvmProgram) -> Result<()> {
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



impl Optimizer2 {
	pub fn new() -> Self {
		Self::default()
	}
	pub fn apply(self, program: &mut LlvmProgram) -> Result<()> {
		// 需在表达式重排前进行，否则，运算指令分布在不同的基本块中， LER做不了任何事情
		RemoveDeadCode::new().apply(program)?;
		RemoveUselessCode::new().apply(program)?;
		RemoveUnreachCode::new().apply(program)?;

		LocalExpressionRearrangement::new().apply(program)?;
		RemoveUselessCode::new().apply(program)?;
		loop {
			let mut flag = false;
			flag |= RemoveDeadCode::new().apply(program)?;
			flag |= RemoveUselessCode::new().apply(program)?;
			flag |= RemoveUnreachCode::new().apply(program)?;

			if let Ok(val) = std::env::var("beta") {
				if val != "n" {
					flag |= FuyukiLocalValueNumber::new().apply(program)?;
				}
			}
			if !flag {
				break;
			}
		}
		program.analysis();
		Ok(())
	}
}
