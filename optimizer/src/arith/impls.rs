use std::{borrow::BorrowMut, collections::HashMap, fmt::Write, usize};

use super::ArithSimplify;
use crate::{metadata::MetaData, RrvmOptimizer};
use llvm::{LlvmTemp, Value, VarType};
use rrvm::{dominator::compute_dominator, program::LlvmProgram, LlvmCFG, LlvmNode};
use utils::errors::Result;

impl RrvmOptimizer for ArithSimplify{
	fn new() -> Self {
		Self{}
	}

	fn apply(self, program: &mut LlvmProgram, _meta: &mut MetaData) -> Result<bool> {
		let mut changed = false;
		program.analysis();

		for func in program.funcs.iter_mut(){
			for bb in &mut func.cfg.blocks {
				
			}
		}
		// todo!();

		Ok(changed)
	}
}