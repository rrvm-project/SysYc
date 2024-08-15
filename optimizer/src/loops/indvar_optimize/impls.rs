use llvm::{LlvmTemp, LlvmTempManager};
use rrvm::{program::LlvmFunc, rrvm_loop::LoopPtr, LlvmNode};

use crate::{loops::loop_data::LoopData, metadata::FuncData};

use super::{one_loop_solver::OneLoopSolver, IndvarOptimize};

impl<'a> IndvarOptimize<'a> {
	pub fn new(
		func: &'a mut LlvmFunc,
		loopdata: &'a mut LoopData,
		funcdata: &'a mut FuncData,
		temp_mgr: &'a mut LlvmTempManager,
	) -> Self {
		Self {
			func,
			loopdata,
			funcdata,
			temp_mgr,
		}
	}
	pub fn apply(mut self) -> bool {
		self.dfs(self.loopdata.root_loop.clone())
	}
	fn dfs(&mut self, loop_: LoopPtr) -> bool {
		let mut flag = false;

		// prevent BorrowMutError
		let subloops = loop_.borrow().subloops.clone();
		for l in subloops.into_iter() {
			flag |= self.dfs(l);
		}
		let loop_brw = loop_.borrow();
		// 不 visit root_loop
		if loop_brw.outer.is_none() {
			return flag;
		}
		if let Some(preheader) = loop_brw.get_loop_preheader(
			&loop_brw
				.blocks_without_subloops(&self.func.cfg, &self.loopdata.loop_map),
		) {
			flag |= self.visit_loop(loop_.clone(), preheader);
		}
		flag
	}
	// TODO: 识别变量的 use-def 环; 识别循环不变量; 识别归纳变量; 归纳变量外推
	fn visit_loop(&mut self, loop_: LoopPtr, preheader: LlvmNode) -> bool {
		let mut solver = OneLoopSolver::new(
			self.func,
			self.loopdata,
			self.funcdata,
			self.temp_mgr,
			loop_.clone(),
			preheader,
		);
		let phi_defs: Vec<LlvmTemp> = loop_
			.borrow()
			.header
			.borrow()
			.phi_instrs
			.iter()
			.map(|i| i.target.clone())
			.collect();
		for use_ in phi_defs.iter() {
			solver.run(use_.clone());
		}
		solver.classify_variant();
		solver.get_loop_info();
		solver.loopdata.indvars.extend(solver.indvars);
		solver.flag
	}
}
