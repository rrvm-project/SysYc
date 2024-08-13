use std::collections::HashMap;

use crate::{
	loops::{
		indvar_optimize::IndvarOptimize, loop_data::LoopData,
		loop_simplify::LoopSimplify,
	},
	metadata::{FuncData, MetaData},
};
use llvm::LlvmTempManager;
use rrvm::program::{LlvmFunc, LlvmProgram};
use utils::Result;

use super::HandleLoops;

impl HandleLoops {
	pub fn new(program: &mut LlvmProgram) -> Self {
		let mut loopdatas = HashMap::new();
		fn solve(func: &mut LlvmFunc, loopdatas: &mut HashMap<String, LoopData>) {
			let loopdata = LoopData::new(func);
			loopdatas.insert(func.name.clone(), loopdata);
		}

		program.funcs.iter_mut().for_each(|func| solve(func, &mut loopdatas));
		Self { loopdatas }
	}
	pub fn loop_simplify(
		&mut self,
		program: &mut LlvmProgram,
		metadata: &mut MetaData,
	) -> Result<bool> {
		fn solve(
			func: &mut LlvmFunc,
			loop_data: &mut LoopData,
			func_data: &mut FuncData,
			temp_mgr: &mut LlvmTempManager,
		) -> bool {
			let opter = LoopSimplify::new(func, loop_data, func_data, temp_mgr);
			opter.apply()
		}

		Ok(program.funcs.iter_mut().fold(false, |last, func| {
			solve(
				func,
				self.loopdatas.get_mut(&func.name).unwrap(),
				metadata.get_func_data(&func.name),
				&mut program.temp_mgr,
			) || last
		}))
	}
	pub fn indvar_optimize(
		&mut self,
		program: &mut LlvmProgram,
		metadata: &mut MetaData,
	) -> Result<bool> {
		fn solve(
			func: &mut LlvmFunc,
			loop_data: &mut LoopData,
			func_data: &mut FuncData,
			temp_mgr: &mut LlvmTempManager,
		) -> bool {
			let opter = IndvarOptimize::new(func, loop_data, func_data, temp_mgr);
			opter.apply()
		}

		Ok(program.funcs.iter_mut().fold(false, |last, func| {
			solve(
				func,
				self.loopdatas.get_mut(&func.name).unwrap(),
				metadata.get_func_data(&func.name),
				&mut program.temp_mgr,
			) || last
		}))
	}
}
