use rrvm::{rrvm_loop::LoopPtr, LlvmCFG};

use super::{loopinfo::LoopInfo, LoopOptimizer};

impl LoopOptimizer {
	pub fn new() -> Self {
		Self {
			temp_graph: super::TempGraph::new(),
		}
	}

	pub fn apply(&mut self, loop_: LoopPtr, cfg: &LlvmCFG) -> bool {
		let mut flag = false;
		self.build_graph(cfg);
		flag |= self.bfs(loop_);
		flag
	}

	fn bfs(&mut self, loop_: LoopPtr) -> bool {
		let mut flag = false;
		for l in loop_.borrow().subloops.iter() {
			flag |= self.bfs(l.clone());
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
}
