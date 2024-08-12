// Ref: https://blog.csdn.net/Dong_HFUT/article/details/121510224

use std::collections::HashMap;

use utils::{InstrTrait, TempTrait};

use crate::cfg::{Node, CFG};

pub fn compute_dominator_frontier<T: InstrTrait<U>, U: TempTrait>(
	cfg: &CFG<T, U>,
	reverse: bool,
	dominates: &HashMap<i32, Vec<Node<T, U>>>,
	dominator: &HashMap<i32, Node<T, U>>,
	dominator_frontier: &mut HashMap<i32, Vec<Node<T, U>>>,
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
			for pred in bb.borrow().prev.iter() {
				let mut runner = pred.clone();
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
