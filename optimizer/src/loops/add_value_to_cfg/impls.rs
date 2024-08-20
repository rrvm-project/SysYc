use llvm::LlvmTempManager;
use rrvm::{dominator::LlvmDomTree, program::LlvmFunc, rrvm_loop::LoopPtr};

use crate::{loops::loop_data::LoopData, metadata::FuncData};

use super::{one_loop_solver::OneLoopSolver, AddValuetoCfg};

impl<'a> AddValuetoCfg<'a> {
	pub fn new(
		func: &'a mut LlvmFunc,
		loopdata: &'a mut LoopData,
		funcdata: &'a mut FuncData,
		temp_mgr: &'a mut LlvmTempManager,
		dom_tree: LlvmDomTree,
	) -> Self {
		Self {
			func,
			loopdata,
			funcdata,
			temp_mgr,
			dom_tree,
		}
	}
	pub fn apply(mut self) -> bool {
		self.loopdata.rebuild(self.func);
		self.dfs(self.loopdata.root_loop.clone())
	}
	// 返回自己是否做出优化，以及汇报自己用了哪些外层循环的变量, 自己的 phi 语句数量和子循环中 phi 语句数量的最大值之和
	fn dfs(&mut self, loop_: LoopPtr) -> bool {
		let mut flag = false;

		// prevent BorrowMutError
		let subloops = loop_.borrow().subloops.clone();
		// 收集子循环都用了哪些外层循环的变量
		for l in subloops.into_iter() {
			flag |= self.dfs(l.clone())
		}
		// 不 visit root_loop
		if loop_.borrow().outer.is_none() {
			return flag;
		}
		flag |= self.visit_loop(loop_.clone());
		flag
	}
	// TODO: 识别变量的 use-def 环; 识别循环不变量; 识别归纳变量; 归纳变量外推
	fn visit_loop(&mut self, loop_: LoopPtr) -> bool {
		let mut solver = OneLoopSolver::new(
			self.func,
			self.loopdata,
			self.funcdata,
			self.temp_mgr,
			&self.dom_tree,
			loop_.clone(),
		);
		solver.classify_indvar();
		for (_, iv) in solver.indvars.clone() {
			if let Some(t) = iv.base.unwrap_temp() {
				solver.place_temp_into_cfg(&t);
			}
		}
		solver.loopdata.indvars.extend(solver.indvars.clone());
		solver.flag
	}
}
