use std::collections::{HashMap, HashSet, VecDeque};

use llvm::{LlvmTemp, LlvmTempManager, Value};
use rrvm::{program::LlvmFunc, rrvm_loop::LoopPtr, LlvmCFG, LlvmNode};

use super::temp_graph::TempGraph;

pub struct LoopOptimizer {
	// 从自己指向自己的 use
	pub temp_graph: TempGraph,
	// 每个 basicblock 属于哪个循环
	pub loop_map: HashMap<i32, LoopPtr>,
	// 每个变量在哪个基本块中被定义
	pub def_map: HashMap<LlvmTemp, LlvmNode>,
}

impl LoopOptimizer {
	pub fn new() -> Self {
		Self {
			temp_graph: TempGraph::new(),
			loop_map: HashMap::new(),
			def_map: HashMap::new(),
		}
	}

	pub fn apply(
		&mut self,
		root_loop: LoopPtr,
		func: &mut LlvmFunc,
		temp_mgr: &mut LlvmTempManager,
	) -> bool {
		let mut flag = false;
		flag |= self.simplify_loop(root_loop.clone(), func, temp_mgr);
		self.build(&func.cfg);
		flag |= self.dfs(root_loop);
		flag
	}

	// 构造处理循环所需要的信息
	pub fn build(&mut self, cfg: &LlvmCFG) {
		self.build_def_map(cfg);
		self.build_graph(cfg);
	}

	fn build_def_map(&mut self, cfg: &LlvmCFG) {
		for bb in cfg.blocks.iter() {
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
		for l in loop_.borrow().subloops.iter() {
			flag |= self.dfs(l.clone());
		}
		flag |= self.visit_loop(loop_);
		flag
	}

	// TODO: 识别变量的 use-def 环; 识别循环不变量; 识别归纳变量; 归纳变量外推
	fn visit_loop(&mut self, loop_: LoopPtr) -> bool {
		let mut invariant: HashSet<LlvmTemp> = HashSet::new();
		let mut variant: HashSet<LlvmTemp> = HashSet::new();
		let _flag = false;

		for phi_def in loop_.borrow().header.borrow().phi_defs.iter() {
			self.bfs_temp(phi_def.clone(), &mut invariant, &mut variant);
		}

		_flag
	}

	#[allow(unused)]
	// 从一个 header 中 phi 语句定义的变量出发，bfs 寻找变量的 use-def 环
	fn bfs_temp(
		&mut self,
		phi_def: LlvmTemp,
		invariant: &mut HashSet<LlvmTemp>,
		variant: &mut HashSet<LlvmTemp>,
	) {
		let mut queue = VecDeque::new();
		queue.push_back(phi_def.clone());
		let mut is_indvar = true;
	}

	#[allow(unused)]
	fn classify_phi_def(&self, phi_def: LlvmTemp) -> Option<(Value, Value)> {
		None
	}
}
