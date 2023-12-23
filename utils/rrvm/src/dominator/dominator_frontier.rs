// Ref: https://blog.csdn.net/Dong_HFUT/article/details/121510224

use crate::{LlvmCFG, LlvmNode};
use std::collections::HashMap;

pub fn compute_dominator_frontier(
	cfg: &mut LlvmCFG,
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
						// runner.dominance_frontier.borrow_mut().push(bb.clone());
						// let runner_dominator =
						// 	runner.dominator.borrow().as_ref().unwrap().clone();
						// runner = runner_dominator;
					}
				}
			}
		} else if bb.borrow().prev.len() > 1 {
			for pred in bb.borrow().prev.iter() {
				let mut runner = pred.clone();
				let mut runner_id = runner.borrow().id;
				while !(dominates.get(&runner_id).map_or(false, |v| v.contains(bb))
					&& runner_id != bb.borrow().id)
				{
					dominator_frontier.entry(runner_id).or_default().push(bb.clone());
					runner = dominator.get(&runner_id).cloned().unwrap();
					runner_id = runner.borrow().id;
					// runner.dominance_frontier.borrow_mut().push(bb.clone());
					// let runner_dominator =
					// 	runner.dominator.borrow().as_ref().unwrap().clone();
					// runner = runner_dominator;
				}
			}
		}
	}

	println!("hello");
	dominator_frontier.iter().for_each(|(k, v)| {
		print!("dominator frontier {}: ", k);
		v.iter().for_each(|x| print!("{}, ", x.borrow().id));
		println!();
	});
}
