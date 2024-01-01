use rrvm::{program::LlvmProgram, LlvmCFG};
use utils::Result;

use crate::{strength_reduce::osr::OSR, RrvmOptimizer};

use super::StrengthReduce;

impl RrvmOptimizer for StrengthReduce {
	fn new() -> Self {
		Self { total_new_temp: 0 }
	}
	fn apply(self, _program: &mut LlvmProgram) -> Result<bool> {
		unimplemented!()
	}
}

impl StrengthReduce {
	pub fn new_with_total_new_temp(total_new_temp: u32) -> Self {
		Self { total_new_temp }
	}
	// 把 total_new_temp 也返回出去
	pub fn apply_strength_reduce(
		self,
		program: &mut LlvmProgram,
	) -> Result<(bool, u32)> {
		let solve = |cfg: &mut LlvmCFG, total_new_temp| -> (bool, u32) {
			let mut osr = OSR::new(cfg, total_new_temp);
			osr.run(cfg);
			(osr.flag, osr.total_new_temp)
		};

		Ok(program.funcs.iter_mut().fold(
			(false, self.total_new_temp),
			|last, func| {
				let (new_flag, new_total) = solve(&mut func.cfg, last.1);
				(last.0 || new_flag, new_total)
			},
		))
	}
}
