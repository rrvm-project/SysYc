use std::{cell::RefCell, rc::Rc};

use crate::{LlvmCFG, LlvmNode};

use super::LoopPtr;

impl LlvmCFG {
	pub fn loop_analysis(&mut self) -> Vec<LoopPtr> {
		self.compute_dominator();
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
// 而它常常是 borrow_mut 的，导致在函数体内无法对自己 borrow，而在函数体内是会经常碰到要 borrow 自己的时候的
pub fn loop_dfs(cur_bb: LlvmNode, cfg: &LlvmCFG) {
	// dfs on dom tree
	cur_bb.borrow_mut().loop_ = None;
	for next in cur_bb.borrow().dominates_directly.iter() {
		loop_dfs(next.clone(), cfg);
	}
	let mut bbs = Vec::new();
	// 看看自己的前驱有没有被自己支配的，有的话就有循环存在，与自己前驱之间的边就是 backedge
	for prev in cur_bb.borrow().prev.iter() {
		if cur_bb.borrow().dominates.contains(prev) {
			bbs.push(prev.clone());
		}
	}
	if !bbs.is_empty() {
		// 这里需要一个指向 cur_bb 的 Rc，我不能通过 Rc::new 来创建，只得把 cfg 也拉过来，从 cfg 里面复制
		let ptr_to_self = cfg
			.blocks
			.iter()
			.find(|bb| bb.borrow().id == cur_bb.borrow().id)
			.unwrap()
			.clone();
		let new_loop = Rc::new(RefCell::new(super::Loop::new(ptr_to_self)));
		while let Some(bb) = bbs.pop() {
			if bb.borrow().loop_.is_none() {
				bb.borrow_mut().loop_ = Some(new_loop.clone());
				if bb.borrow().id != cur_bb.borrow().id {
					bbs.append(bb.borrow().prev.clone().as_mut());
				}
			} else {
				let mut inner_loop = bb.borrow().loop_.clone().unwrap();
				let mut outer_loop = inner_loop.borrow().outer.clone();
				while let Some(outer) = outer_loop.clone() {
					inner_loop = outer;
					outer_loop = inner_loop.borrow().outer.clone();
				}
				if inner_loop == new_loop {
					continue;
				}
				new_loop.borrow_mut().no_inner = false;
				inner_loop.borrow_mut().outer = Some(new_loop.clone());
				bbs.append(inner_loop.borrow().header.borrow().prev.clone().as_mut());
			}
		}
	}
}
