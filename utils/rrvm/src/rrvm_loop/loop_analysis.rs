use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::super::{dominator::compute_dominator, LlvmCFG, LlvmNode};

use super::{Loop, LoopPtr};

impl LlvmCFG {
	pub fn loop_analysis(
		&mut self,
		loop_map: &mut HashMap<i32, LoopPtr>,
	) -> LoopPtr {
		let mut dominates: HashMap<i32, Vec<LlvmNode>> = HashMap::new();
		let mut dominates_directly: HashMap<i32, Vec<LlvmNode>> = HashMap::new();
		let mut dominator: HashMap<i32, LlvmNode> = HashMap::new();
		compute_dominator(
			self,
			false,
			&mut dominates,
			&mut dominates_directly,
			&mut dominator,
		);
		loop_dfs(
			self.get_entry(),
			self,
			loop_map,
			&dominates,
			&dominates_directly,
		);
		// 计算每一个 loop 的深度
		for bb in self.blocks.iter() {
			calc_loop_level(loop_map.get(&bb.borrow().id).cloned());
		}
		// 创造 loop tree 的根节点，也就是代表整个控制流的那个，只执行一次的 loop
		let root_loop = Rc::new(RefCell::new(Loop::new(self.get_entry())));
		root_loop.borrow_mut().level = 0;
		let mut cur_id = 1;
		for bb in self.blocks.iter() {
			if let Some(l) = loop_map.get(&bb.borrow().id).cloned() {
				if l.borrow().id == 0 {
					l.borrow_mut().id = cur_id;
					cur_id += 1;
				}
				if l.borrow().outer.is_none() {
					root_loop.borrow_mut().subloops.push(l.clone());
					l.borrow_mut().outer.replace(root_loop.clone());
				}
			}
		}
		root_loop
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
pub fn loop_dfs(
	cur_bb: LlvmNode,
	_cfg: &LlvmCFG,
	loop_map: &mut HashMap<i32, LoopPtr>,
	dominates: &HashMap<i32, Vec<LlvmNode>>,
	dominates_directly: &HashMap<i32, Vec<LlvmNode>>,
) {
	// dfs on dom tree
	// 换成 hashmap 存储后，就不用标记成 None 了
	// cur_bb.borrow_mut().loop_ = None;
	let cur_bb_id = cur_bb.borrow().id;
	for next in
		dominates_directly.get(&cur_bb_id).cloned().unwrap_or_default().iter()
	{
		loop_dfs(next.clone(), _cfg, loop_map, dominates, dominates_directly);
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
			if loop_map.get(&bb.borrow().id).is_none() {
				loop_map.insert(bb.borrow().id, new_loop.clone());
				new_loop.borrow_mut().blocks.push(bb.clone());
				if bb.borrow().id != cur_bb.borrow().id {
					bbs.append(bb.borrow().prev.clone().as_mut());
				}
			} else {
				let mut inner_loop = loop_map.get(&bb.borrow().id).cloned().unwrap();
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
