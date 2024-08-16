use std::collections::HashSet;

use llvm::{LlvmTemp, LlvmTempManager};
use rrvm::{dominator::LlvmDomTree, program::LlvmFunc, rrvm_loop::LoopPtr};

use crate::{loops::loop_data::LoopData, metadata::FuncData};

use super::{one_loop_solver::OneLoopSolver, IndvarExtraction};

impl<'a> IndvarExtraction<'a> {
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
		let (flag, _) = self.dfs(self.loopdata.root_loop.clone());
		flag
	}
	// 返回自己是否做出优化，以及汇报自己用了哪些外层循环的变量
	fn dfs(&mut self, loop_: LoopPtr) -> (bool, HashSet<LlvmTemp>) {
		let mut flag = false;
		let mut outside_use = HashSet::new();

		// prevent BorrowMutError
		let subloops = loop_.borrow().subloops.clone();
		// 收集子循环都用了哪些外层循环的变量
		for l in subloops.into_iter() {
			let (subloop_flag, subloop_outside_use) = self.dfs(l);
			flag |= subloop_flag;
			outside_use.extend(subloop_outside_use);
		}
		// 不 visit root_loop
		if loop_.borrow().outer.is_none() {
			return (flag, HashSet::new());
		}
		flag |= self.visit_loop(loop_.clone(), &mut outside_use);
		outside_use.retain(|temp| {
			!self.loopdata.def_map.get(temp).is_some_and(|def| {
				self.loopdata.loop_map[&def.borrow().id].borrow().id
					== loop_.borrow().id
			})
		});
		(flag, outside_use)
	}
	// TODO: 识别变量的 use-def 环; 识别循环不变量; 识别归纳变量; 归纳变量外推
	fn visit_loop(
		&mut self,
		loop_: LoopPtr,
		outside_use: &mut HashSet<LlvmTemp>,
	) -> bool {
		let mut solver = OneLoopSolver::new(
			self.func,
			self.loopdata,
			self.funcdata,
			self.temp_mgr,
			outside_use,
			&self.dom_tree,
			loop_.clone(),
		);
		solver.classify_indvar();
		solver.indvar_extraction();
		solver.loopdata.indvars.extend(solver.indvars);
		solver.flag
	}
}
