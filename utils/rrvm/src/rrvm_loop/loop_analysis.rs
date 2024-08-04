use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::super::{dominator::compute_dominator, LlvmCFG, LlvmNode};

use super::LoopPtr;

impl LlvmCFG {
	pub fn loop_analysis(&mut self) -> Vec<LoopPtr> {
		loop_dfs(self.get_entry(), self);
		for bb in self.blocks.iter() {
			calc_loop_level(bb.borrow().loop_.clone());
		}
		// 收集所有的 loop
		let mut loops = Vec::new();
		for bb in self.blocks.iter() {
			if let Some(loop_) = bb.borrow().loop_.clone() {
				if !loops.contains(&loop_) {
					loops.push(loop_);
				}
			}
		}
		loops
	}
}

fn calc_loop_level(loop_: Option<LoopPtr>) {
	if let Some(l) = loop_ {
		if l.borrow().level != -1 {
			return;
		}
		let outer = l.borrow().outer.clone();
		if let Some(outer) = outer {
			calc_loop_level(Some(outer.clone()));
			l.borrow_mut().level = outer.borrow().level + 1;
		} else {
			l.borrow_mut().level = 1;
		}
	}
}

// 这里本来想实现成 LlvmNode 的一个成员函数的，但这样做，参数中就会有一个 &mut self,
// 而它常常是一个 borrow_mut 的结果，这导致在函数体内无法再对自己 borrow。
pub fn loop_dfs(cur_bb: LlvmNode, cfg: &LlvmCFG) {
	let mut dominates: HashMap<i32, Vec<LlvmNode>> = HashMap::new();
	let mut dominates_directly: HashMap<i32, Vec<LlvmNode>> = HashMap::new();
	let mut dominator: HashMap<i32, LlvmNode> = HashMap::new();
	compute_dominator(
		cfg,
		true,
		&mut dominates,
		&mut dominates_directly,
		&mut dominator,
	);

	// dfs on dom tree
	cur_bb.borrow_mut().loop_ = None;
	let cur_bb_id = cur_bb.borrow().id;
	for next in
		dominates_directly.get(&cur_bb_id).cloned().unwrap_or_default().iter()
	{
		loop_dfs(next.clone(), cfg);
	}
	let mut bbs = Vec::new();
	// 看看自己的前驱有没有被自己支配的，有的话就有循环存在，与自己前驱之间的边就是 backedge
	for prev in cur_bb.borrow().prev.iter() {
		if dominates.get(&cur_bb_id).cloned().unwrap_or_default().contains(prev) {
			bbs.push(prev.clone());
		}
	}
	if !bbs.is_empty() {
		let new_loop = Rc::new(RefCell::new(super::Loop::new(cur_bb.clone())));
		new_loop.borrow_mut().blocks.push(cur_bb.clone());
		while let Some(bb) = bbs.pop() {
			if bb.borrow().loop_.is_none() {
				bb.borrow_mut().loop_ = Some(new_loop.clone());
				new_loop.borrow_mut().blocks.push(bb.clone());
				if bb.borrow().id != cur_bb.borrow().id {
					bbs.append(bb.borrow().prev.clone().as_mut());
				}
			} else {
				let mut inner_loop = bb.borrow().loop_.clone().unwrap();
				let mut outer_loop = inner_loop.borrow().outer.clone();
				while let Some(outer) = outer_loop.clone() {
					inner_loop = outer;
					outer_loop.clone_from(&inner_loop.borrow().outer);
				}
				if inner_loop == new_loop {
					continue;
				}
				new_loop.borrow_mut().subloops.push(inner_loop.clone());
				inner_loop.borrow_mut().outer = Some(new_loop.clone());
				bbs.append(inner_loop.borrow().header.borrow().prev.clone().as_mut());
			}
		}
	}
}
