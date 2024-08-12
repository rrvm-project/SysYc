// naive algorithm computing dominator tree with complexity O(n*m)
// Ref: https://blog.csdn.net/Dong_HFUT/article/details/121375025?spm=1001.2014.3001.5501

use std::{
	cell::RefCell,
	collections::{HashMap, HashSet, VecDeque},
	rc::Rc,
};

use utils::{InstrTrait, TempTrait};

use crate::{
	basicblock::BasicBlock,
	cfg::{Node, CFG},
};

// 如果要计算反向支配树，计算dominates时可能需要创建一个假的出口节点，但计算dominator和dominates_directly时会将这个假的出口节点排除在外，这会导致部分节点没有dominator
pub fn compute_dominator<T: InstrTrait<U>, U: TempTrait>(
	cfg: &CFG<T, U>,
	reverse: bool,
	dominates: &mut HashMap<i32, Vec<Node<T, U>>>,
	dominates_directly: &mut HashMap<i32, Vec<Node<T, U>>>,
	dominator: &mut HashMap<i32, Node<T, U>>,
) {
	let mut block_has_ret = Vec::new();
	for bb in cfg.blocks.iter() {
		if bb.borrow().succ.is_empty() {
			block_has_ret.push(bb.clone());
		}
	}
	let exit;
	if block_has_ret.len() == 1 {
		exit = block_has_ret[0].clone();
	} else {
		exit = Rc::new(RefCell::new(BasicBlock::new(-1, 0.0)));
		block_has_ret.iter().for_each(|bb| {
			bb.borrow_mut().succ.push(exit.clone());
			exit.borrow_mut().prev.push(bb.clone());
		});
	}
	for bb in cfg.blocks.iter() {
		// 尝试将这个 bb 从图中移除，移除后无法访问的节点是被它支配的节点
		let to_be_removed = bb.borrow().id;

		let mut reachable = HashSet::new();
		let mut worklist = VecDeque::new();
		if reverse {
			if to_be_removed != exit.borrow().id {
				worklist.push_back(exit.clone());
			}
		} else if to_be_removed != cfg.get_entry().borrow().id {
			worklist.push_back(cfg.get_entry().clone());
		}
		while let Some(bb) = worklist.pop_front() {
			if reachable.contains(&bb.borrow().id) {
				continue;
			}
			reachable.insert(bb.borrow().id);
			if reverse {
				for pred in bb.borrow().prev.iter() {
					if pred.borrow().id != to_be_removed {
						worklist.push_back(pred.clone());
					}
				}
			} else {
				for succ in bb.borrow().succ.iter() {
					if succ.borrow().id != to_be_removed {
						worklist.push_back(succ.clone());
					}
				}
			}
		}
		cfg.blocks.iter().for_each(|bb_inner| {
			if !reachable.contains(&bb_inner.borrow().id) {
				dominates.entry(bb.borrow().id).or_default().push(bb_inner.clone());
			}
		});
	}
	// 计算完dominates后，计算dominates_directly
	for bb in cfg.blocks.iter() {
		let bb_id = bb.borrow().id;
		dominates[&bb_id].iter().for_each(|bb_inner| {
			let bb_inner_id = bb_inner.borrow().id;
			if bb_inner_id == bb_id {
				return;
			}
			// 如果bb_inner没有支配者
			if dominator.get(&bb_inner_id).is_none() {
				dominates_directly.entry(bb_id).or_default().push(bb_inner.clone());
				dominator.insert(bb_inner_id, bb.clone());
			// 如果bb_inner的支配者支配了bb
			} else if dominates
				[&dominator.get(&bb_inner_id).as_ref().unwrap().borrow().id]
				.contains(bb)
			{
				dominates_directly.entry(bb_id).or_default().push(bb_inner.clone());
				// TODO: 这里需要把bb_inner 从原来的直接支配者的直接支配集合中去掉，有没有比retain更好的方法？
				dominates_directly
					.entry(dominator.get(&bb_inner_id).as_ref().unwrap().borrow().id)
					.or_default()
					.retain(|x| x.borrow().id != bb_inner_id);
				dominator.insert(bb_inner_id, bb.clone());
			}
		});
	}
	block_has_ret.iter().for_each(|bb| {
		bb.borrow_mut().succ.clear();
	});
}
