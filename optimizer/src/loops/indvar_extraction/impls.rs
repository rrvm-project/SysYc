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
		self.loopdata.rebuild(self.func);
		let (flag, _, _phi_num) = self.dfs(self.loopdata.root_loop.clone());
		#[cfg(feature = "debug")]
		eprintln!("loop: entry, subloop phi num: {}", _phi_num);

		// let missing = LlvmTemp{
		// 	name: "128".into(),
		// 	is_global: false,
		// 	var_type: llvm::VarType::I32Ptr,
		// };
		// dbg!(self.loopdata.indvars.get(&missing));

		flag
	}
	// 返回自己是否做出优化，以及汇报自己用了哪些外层循环的变量, 自己的 phi 语句数量和子循环中 phi 语句数量的最大值之和
	fn dfs(&mut self, loop_: LoopPtr) -> (bool, HashSet<LlvmTemp>, usize) {
		let mut flag = false;
		let mut outside_use = HashSet::new();
		let mut phi_num = 0;

		// prevent BorrowMutError
		let subloops = loop_.borrow().subloops.clone();
		// 收集子循环都用了哪些外层循环的变量
		for l in subloops.into_iter() {
			let (subloop_flag, subloop_outside_use, subloop_phi_num) =
				self.dfs(l.clone());
			flag |= subloop_flag;
			outside_use.extend(subloop_outside_use);
			if phi_num < subloop_phi_num {
				phi_num = subloop_phi_num;
			}
		}
		// 不 visit root_loop
		if loop_.borrow().outer.is_none() {
			return (flag, HashSet::new(), phi_num);
		}
		flag |= self.visit_loop(loop_.clone(), &mut outside_use, phi_num);
		phi_num += loop_.borrow().header.borrow().phi_instrs.len();
		outside_use.retain(|temp| {
			!self.loopdata.def_map.get(temp).is_some_and(|def| {
				self.loopdata.loop_map[&def.borrow().id].borrow().id
					== loop_.borrow().id
			})
		});
		(flag, outside_use, phi_num)
	}
	// TODO: 识别变量的 use-def 环; 识别循环不变量; 识别归纳变量; 归纳变量外推
	fn visit_loop(
		&mut self,
		loop_: LoopPtr,
		outside_use: &mut HashSet<LlvmTemp>,
		phi_num: usize,
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
		solver.indvar_extraction(phi_num);
		solver.loopdata.indvars.extend(solver.indvars);
		solver.flag
	}
}
