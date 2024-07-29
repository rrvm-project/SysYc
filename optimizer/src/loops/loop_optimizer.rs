use std::collections::HashMap;

use llvm::LlvmTemp;
use rrvm::{rrvm_loop::LoopPtr, LlvmCFG};

use super::{loopinfo::LoopInfo, LoopOptimizer};

impl LoopOptimizer {
	pub fn new() -> Self {
		Self {
			temp_graph: super::TempGraph::new(),
			loop_map: HashMap::new(),
		}
	}

	pub fn apply(&mut self, loop_: LoopPtr, cfg: &LlvmCFG) -> bool {
		let mut flag = false;
		self.build_graph(cfg);
		// println!("{}", self.temp_graph);
		flag |= self.dfs(loop_);
		flag
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
	fn visit_loop(&mut self, _loop_: LoopPtr) -> bool {
		let _flag = false;
		let mut _info = LoopInfo::new();
		_flag
	}

	#[allow(unused)]
	fn dfs_temp(&mut self, temp: LlvmTemp) {}
}
