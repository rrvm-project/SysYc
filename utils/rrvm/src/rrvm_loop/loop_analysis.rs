use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::super::{dominator::compute_dominator, LlvmCFG, LlvmNode};

<<<<<<< HEAD
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
		// 创造 loop tree 的根节点，也就是代表整个控制流的那个，只执行一次的 loop
		let root_loop = Rc::new(RefCell::new(Loop::new(self.get_entry())));

		loop_dfs(self.get_entry(), loop_map, &dominates, &dominates_directly);

=======
use super::LoopPtr;

impl LlvmCFG {
	pub fn loop_analysis(&mut self) -> Vec<LoopPtr> {
		loop_dfs(self.get_entry(), self);
>>>>>>> 6506c1f (feat: kill stack array)
		for bb in self.blocks.iter() {
			if let Some(l) = loop_map.get(&bb.borrow().id).cloned() {
				if l.borrow().outer.is_none() {
					root_loop.borrow_mut().subloops.push(l.clone());
					l.borrow_mut()
						.outer
						.replace(Rc::<RefCell<Loop>>::downgrade(&root_loop));
				}
			} else {
				loop_map.insert(bb.borrow().id, root_loop.clone());
			}
		}

		let mut dfs_clock = 0;

		// Thus we can look up whether loop A is (indirect) subloop of loop B O(1)
		fn dfs_loop_tree(dfs_clock: &mut u32, current: &LoopPtr, depth: i32) {
			*dfs_clock += 1;
			current.borrow_mut().id = *dfs_clock;
			current.borrow_mut().level = depth;
			for sub in &current.borrow().subloops {
				dfs_loop_tree(dfs_clock, sub, depth + 1)
			}
			*dfs_clock += 1;
			current.borrow_mut().ura_id = *dfs_clock;
		}

		dfs_loop_tree(&mut dfs_clock, &root_loop, 0);

		root_loop
	}
}

// 这里本来想实现成 LlvmNode 的一个成员函数的，但这样做，参数中就会有一个 &mut self,
// 而它常常是一个 borrow_mut 的结果，这导致在函数体内无法再对自己 borrow。
<<<<<<< HEAD
pub fn loop_dfs(
	cur_bb: LlvmNode,
	loop_map: &mut HashMap<i32, LoopPtr>,
	dominates: &HashMap<i32, Vec<LlvmNode>>,
	dominates_directly: &HashMap<i32, Vec<LlvmNode>>,
) {
=======
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

>>>>>>> 6506c1f (feat: kill stack array)
	// dfs on dom tree
	// 换成 hashmap 存储后，就不用标记成 None 了
	// cur_bb.borrow_mut().loop_ = None;
	let cur_bb_id = cur_bb.borrow().id;
	for next in
		dominates_directly.get(&cur_bb_id).cloned().unwrap_or_default().iter()
	{
<<<<<<< HEAD
		loop_dfs(next.clone(), loop_map, dominates, dominates_directly);
=======
		loop_dfs(next.clone(), cfg);
>>>>>>> 6506c1f (feat: kill stack array)
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

		while let Some(bb) = bbs.pop() {
			if loop_map.get(&bb.borrow().id).is_none() {
				loop_map.insert(bb.borrow().id, new_loop.clone());

				if bb.borrow().id != cur_bb.borrow().id {
					bbs.append(bb.borrow().prev.clone().as_mut());
				}
			} else {
				let mut inner_loop = loop_map.get(&bb.borrow().id).cloned().unwrap();
				let mut outer_loop = inner_loop.borrow().outer.clone();
				while let Some(outer) = outer_loop.clone() {
					inner_loop = outer.upgrade().unwrap();
					outer_loop.clone_from(&inner_loop.borrow().outer);
				}
				if inner_loop.borrow().header.borrow().id
					== new_loop.borrow().header.borrow().id
				{
					continue;
				}
				new_loop.borrow_mut().subloops.push(inner_loop.clone());
				inner_loop.borrow_mut().outer =
					Some(Rc::<RefCell<Loop>>::downgrade(&new_loop));
				bbs.append(inner_loop.borrow().header.borrow().prev.clone().as_mut());
			}
		}
	}
}
