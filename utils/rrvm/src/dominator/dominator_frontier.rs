// Ref: https://blog.csdn.net/Dong_HFUT/article/details/121510224

use crate::{LlvmCFG, LlvmNode};
use std::collections::HashMap;

pub fn compute_dominator_frontier(
	cfg: &LlvmCFG,
	reverse: bool,
	dominates: &HashMap<i32, Vec<LlvmNode>>,
	dominator: &HashMap<i32, LlvmNode>,
	dominator_frontier: &mut HashMap<i32, Vec<LlvmNode>>,
) {
	for bb in cfg.blocks.iter() {
		if reverse {
			if bb.borrow().succ.len() > 1 {
				for succ in bb.borrow().succ.iter() {
					let mut runner = succ.clone();
					let mut runner_id = runner.borrow().id;
					while !(dominates.get(&runner_id).map_or(false, |v| v.contains(bb))
						&& runner_id != bb.borrow().id)
					{
						dominator_frontier.entry(runner_id).or_default().push(bb.clone());
						if let Some(d) = dominator.get(&runner_id) {
							runner = d.clone();
						} else {
							break;
						}
						runner_id = runner.borrow().id;
					}
				}
			}
		} else if bb.borrow().prev.len() > 1 {
			for prev in bb.borrow().prev.iter() {
				let mut runner = prev.clone();
				let mut runner_id = runner.borrow().id;
				while !(dominates.get(&runner_id).map_or(false, |v| v.contains(bb))
					&& runner_id != bb.borrow().id)
				{
					dominator_frontier.entry(runner_id).or_default().push(bb.clone());
					runner = dominator.get(&runner_id).cloned().unwrap();
					runner_id = runner.borrow().id;
				}
			}
		}
	}
}

impl LlvmCFG {
	// 计算正向支配边界, 将结果存储在每个节点中
	pub fn compute_dominate_frontier(&mut self) {
		for bb in self.blocks.iter() {
			if bb.borrow().prev.len() > 1 {
				for prev in bb.borrow().prev.iter() {
					let mut runner = prev.clone();
					let mut runner_id = runner.borrow().id;
					while !(runner.borrow().dominates.contains(bb)
						&& runner_id != bb.borrow().id)
					{
						runner.borrow_mut().dominate_frontier.push(bb.clone());
						let new_runner = runner.borrow().dominator.clone().unwrap();
						runner = new_runner;
						runner_id = runner.borrow().id;
					}
				}
			}
		}
	}
}
