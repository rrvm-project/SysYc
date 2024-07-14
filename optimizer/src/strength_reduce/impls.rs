use llvm::{LlvmTemp, LlvmTempManager};
use rrvm::{program::LlvmProgram, LlvmCFG};
use utils::Result;

use crate::{metadata::MetaData, strength_reduce::osr::OSR, RrvmOptimizer};

use super::StrengthReduce;

impl RrvmOptimizer for StrengthReduce {
	fn new() -> Self {
		Self {}
	}
	fn apply(
		self,
		program: &mut LlvmProgram,
		_metadata: &mut MetaData,
	) -> Result<bool> {
		let solve = |cfg: &mut LlvmCFG,
		             params: Vec<LlvmTemp>,
		             mgr: &mut LlvmTempManager|
		 -> bool {
			let mut osr = OSR::new(cfg, params);
			osr.run(cfg, mgr);
			osr.flag
		};

		Ok(program.funcs.iter_mut().fold(false, |last, func| {
			let new_flag = solve(
				&mut func.cfg,
				func.params.clone().iter().map(|v| v.unwrap_temp().unwrap()).collect(),
				&mut program.temp_mgr,
			);
			last || new_flag
		}))
	}
}
