use std::collections::HashMap;

use llvm::LlvmTempManager;
use rrvm::program::{LlvmFunc, LlvmProgram};
use utils::errors::Result;

use crate::{
	loops::loop_optimizer::LoopOptimizer,
	metadata::{FuncData, MetaData},
	RrvmOptimizer,
};

use super::HandleLoops;

impl RrvmOptimizer for HandleLoops {
	fn new() -> Self {
		Self {}
	}
	fn apply(
		self,
		program: &mut LlvmProgram,
		metadata: &mut MetaData,
	) -> Result<bool> {
		fn solve(
			func: &mut LlvmFunc,
			temp_mgr: &mut LlvmTempManager,
			funcdata: &mut FuncData,
		) -> bool {
			let mut flag: bool = false;
			let mut loop_map = HashMap::new();
			let root_loop = func.cfg.loop_analysis(&mut loop_map);
			funcdata.loop_map = loop_map.clone();
			let mut opter = LoopOptimizer::new(func, funcdata, temp_mgr);
			opter.loop_map = loop_map;
			// print_all_loops(root_loop.clone());
			flag |= opter.apply(root_loop.clone());
			flag
		}

		Ok(program.funcs.iter_mut().fold(false, |last, func| {
			solve(
				func,
				&mut program.temp_mgr,
				metadata.get_func_data(&func.name),
			) || last
		}))
	}
}
