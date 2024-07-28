// 循指令中 temp 的 use-def 链构造图，每个变量指向它的 use，同时附带上操作名称

use std::collections::HashMap;

use llvm::LlvmTemp;
use rrvm::LlvmCFG;

use super::{LoopOptimizer, OpType, TempGraph};

impl LoopOptimizer {
	pub fn build_graph(&mut self, cfg: &LlvmCFG) {
		for bb in cfg.blocks.iter() {
			for inst in bb.borrow().phi_instrs.iter() {
				let target = inst.target.clone();
				inst.source.iter().for_each(|(temp, _)| {
					self.temp_graph.add_edge(target.clone(), OpType::Phi(temp.clone()));
				});
			}
			for inst in bb.borrow().instrs.iter() {
				if let Some(target) = inst.get_write() {
					inst.get_read_values().into_iter().for_each(|value| {
						self.temp_graph.add_edge(
							target.clone(),
							OpType::from_arithop(inst.get_candidate_operator(), value),
						);
					});
				}
			}
		}
	}
}

impl TempGraph {
	pub fn new() -> Self {
		Self {
			temp_graph: HashMap::new(),
		}
	}

	pub fn add_edge(&mut self, temp: LlvmTemp, op: OpType) {
		self.temp_graph.entry(temp).or_default().push(op);
	}
}
