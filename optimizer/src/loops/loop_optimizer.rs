use std::collections::{HashMap, HashSet};

use llvm::{LlvmTemp, LlvmTempManager};
use rrvm::{program::LlvmFunc, rrvm_loop::LoopPtr, LlvmNode};

use crate::metadata::FuncData;

use super::{indvar_solver::IndVarSolver, temp_graph::TempGraph};

pub struct LoopOptimizer<'a> {
	// 从自己指向自己的 use
	pub temp_graph: TempGraph,
	// 每个 basicblock 属于哪个循环
	pub loop_map: HashMap<i32, LoopPtr>,
	// 每个变量在哪个基本块中被定义
	pub def_map: HashMap<LlvmTemp, LlvmNode>,
	pub funcdata: &'a mut FuncData,
	pub temp_mgr: &'a mut LlvmTempManager,
	pub func: &'a mut LlvmFunc,
	pub temp_params: HashSet<LlvmTemp>,
}

impl<'a> LoopOptimizer<'a> {
	pub fn new(
		func: &'a mut LlvmFunc,
		funcdata: &'a mut FuncData,
		temp_mgr: &'a mut LlvmTempManager,
	) -> Self {
		let params = func
			.params
			.iter()
			.map(|v| {
				v.unwrap_temp().expect("LoopOptimizer: func param is not a temp")
			})
			.collect();
		Self {
			temp_graph: TempGraph::new(),
			loop_map: HashMap::new(),
			def_map: HashMap::new(),
			func,
			funcdata,
			temp_mgr,
			temp_params: params,
		}
	}

	pub fn apply(&mut self, root_loop: LoopPtr) -> bool {
		let mut flag = false;
		self.build();
		flag |= self.simplify_loop(root_loop.clone());
		flag |= self.dfs(root_loop);
		flag
	}

	// 构造处理循环所需要的信息
	pub fn build(&mut self) {
		self.build_def_map();
		self.build_graph();
	}

	fn build_def_map(&mut self) {
		for bb in self.func.cfg.blocks.iter() {
			for inst in bb.borrow().phi_instrs.iter() {
				self.def_map.insert(inst.target.clone(), bb.clone());
			}
			for inst in bb.borrow().instrs.iter() {
				if let Some(target) = inst.get_write() {
					self.def_map.insert(target.clone(), bb.clone());
				}
			}
		}
	}

	fn dfs(&mut self, loop_: LoopPtr) -> bool {
		let mut flag = false;
		// prevent BorrowMutError
		let subloops = loop_.borrow().subloops.clone();
		for l in subloops.into_iter() {
			flag |= self.dfs(l);
		}
		// 不 visit root_loop
		if loop_.borrow().outer.is_none() {
			return flag;
		}
		if let Some(preheader) = loop_.borrow().get_loop_preheader(
			&loop_.borrow().blocks_without_subloops(&self.func.cfg, &self.loop_map),
		) {
			flag |= self.visit_loop(loop_.clone(), preheader);
		}
		flag
	}

	// TODO: 识别变量的 use-def 环; 识别循环不变量; 识别归纳变量; 归纳变量外推
	fn visit_loop(&mut self, loop_: LoopPtr, preheader: LlvmNode) -> bool {
		let mut solver = IndVarSolver::new(
			&mut self.func.cfg,
			self.temp_params.clone(),
			loop_.clone(),
			preheader,
			self.temp_mgr,
			&mut self.temp_graph,
			&mut self.loop_map,
			&mut self.def_map,
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
		solver.move_invariant();
		if let Some(info) = solver.get_loop_info() {
			self.funcdata.loop_infos.insert(loop_.borrow().id, info.clone());
		}
		solver.flag
	}
}
