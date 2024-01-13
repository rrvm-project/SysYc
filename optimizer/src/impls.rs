use rrvm::program::LlvmProgram;
use utils::errors::Result;

use crate::{useless_phis::RemoveUselessPhis, RrvmOptimizer, *};
use dead_code::RemoveDeadCode;
use fuyuki_vn::FuyukiLocalValueNumber;
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
		RemoveDeadCode::new().apply(program)?;
		RemoveUselessCode::new().apply(program)?;
		RemoveUnreachCode::new().apply(program)?;
		RemoveUselessCode::new().apply(program)?;
		loop {
			let mut flag = false;
			flag |= RemoveDeadCode::new().apply(program)?;
			flag |= RemoveUnreachCode::new().apply(program)?;
			flag |= RemoveUselessCode::new().apply(program)?;
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
	pub fn apply(self, program: &mut LlvmProgram) -> Result<()> {
		RemoveDeadCode::new().apply(program)?;
		RemoveUselessCode::new().apply(program)?;
		RemoveUnreachCode::new().apply(program)?;
		RemoveUselessCode::new().apply(program)?;
		loop {
			let mut flag = false;
			flag |= RemoveDeadCode::new().apply(program)?;
			flag |= RemoveUnreachCode::new().apply(program)?;
			flag |= RemoveUselessCode::new().apply(program)?;
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
