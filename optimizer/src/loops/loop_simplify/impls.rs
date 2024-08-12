use std::{
	cell::RefCell,
	collections::{HashMap, HashSet},
	rc::Rc,
};

use llvm::{LlvmInstrTrait, LlvmTemp, PhiInstr, Value};
use rrvm::{
	cfg::{force_link_llvmnode, unlink_node},
	dominator::{compute_dominator, compute_dominator_frontier},
	rrvm_loop::LoopPtr,
	LlvmNode,
};
use utils::Label;

use crate::loops::loop_optimizer::LoopOptimizer;

use super::LoopSimplify;

impl<'a: 'b, 'b> LoopSimplify<'a, 'b> {
	pub fn new(opter: &'b mut LoopOptimizer<'a>) -> Self {
		Self { opter }
	}
	// 按 dfs 序逐个 loop 处理
	pub fn apply(mut self) -> bool {
		let mut flag = false;
		let mut dfs_vec = Vec::new();
		fn dfs(node: LoopPtr, dfs_vec: &mut Vec<LoopPtr>) {
			for subloop in node.borrow().subloops.iter() {
				dfs(subloop.clone(), dfs_vec);
			}
			dfs_vec.push(node);
		}
		dfs(self.opter.root_loop.clone(), &mut dfs_vec);
		// 移去 root_node
		dfs_vec.pop();
		for loop_node in dfs_vec.iter() {
			flag |= self.simplify_one_loop(loop_node.clone());
		}

		let mut dominates: HashMap<i32, Vec<LlvmNode>> = HashMap::new();
		let mut dominates_directly: HashMap<i32, Vec<LlvmNode>> = HashMap::new();
		let mut dominator: HashMap<i32, LlvmNode> = HashMap::new();
		compute_dominator(
			&self.opter.func.cfg,
			false,
			&mut dominates,
			&mut dominates_directly,
			&mut dominator,
		);

		let mut dominator_frontier: HashMap<i32, Vec<LlvmNode>> = HashMap::new();
		compute_dominator_frontier(
			&self.opter.func.cfg,
			false,
			&dominates_directly,
			&dominator,
			&mut dominator_frontier,
		);
		let mut replace_map = HashMap::new();
		for loop_ in dfs_vec.iter() {
			// Scan over the PHI nodes in the loop header.  Since they now have only two
			// incoming values (the loop is canonicalized), we may have simplified the PHI
			// down to 'X = phi [X, Y]', which should be replaced with 'Y'.
			//
			// 若把这一步放到 simplify_one_loop 中做，则需要在 simplify_one_loop 内反复计算支配信息，因为 simplify_one_loop 会改变支配树
			// 而这一步本身不修改支配信息，故拿出来单独做一遍
			self.simplify_header_phis(loop_.clone(), &mut replace_map);
		}
		if !replace_map.is_empty() {
			println!("LoopSimplify: Mapping with {:?}", replace_map);
			flag = true;
			for bb in self.opter.func.cfg.blocks.iter() {
				let mut bb = bb.borrow_mut();
				for phi in bb.phi_instrs.iter_mut() {
					phi.map_temp(&replace_map);
				}
				for inst in bb.instrs.iter_mut() {
					inst.map_temp(&replace_map);
				}
				for jump in bb.jump_instr.iter_mut() {
					jump.map_temp(&replace_map);
				}
			}
		}
		flag
	}
	/// This method introduces at least one new basic block into the function and
	/// moves some of the predecessors of BB to be predecessors of the new block.
	/// The new predecessors are indicated by the Preds array. Returns new basic block to which predecessors
	/// from Preds are now pointing.
	pub fn split_block_predecessors(
		&mut self,
		bb: LlvmNode,
		preds: Vec<LlvmNode>,
		has_loop_exit: bool,
	) -> LlvmNode {
		assert!(!preds.is_empty());

		let new_bb = Rc::new(RefCell::new(self.opter.func.new_basicblock(0.0)));

		// Move the edges from Preds to point to NewBB instead of BB.
		for pred in preds.iter() {
			unlink_node(pred, &bb);
			force_link_llvmnode(pred, &new_bb);
		}

		self.update_phi_nodes(bb.clone(), new_bb.clone(), preds, has_loop_exit);

		force_link_llvmnode(&new_bb, &bb);

		let target_pos =
			self.opter.func.cfg.blocks.iter().position(|v| *v == bb).unwrap();
		self.opter.func.cfg.blocks.insert(target_pos, new_bb.clone());

		new_bb
	}
	pub fn update_phi_nodes(
		&mut self,
		bb: LlvmNode,
		new_bb: LlvmNode,
		preds: Vec<LlvmNode>,
		has_loop_exit: bool, // new_bb 是否是某循环的 exit
	) {
		// Create a new PHI node in NewBB for each PHI node in OrigBB.
		for phi in bb.borrow_mut().phi_instrs.iter_mut() {
			// Check to see if all of the values coming in are the same.  If so, we
			// don't need to create a new PHI node, unless it's needed for LCSSA.
			// MAYBETODO：似乎可以改成 iter().fold() 的形式？？
			let mut in_var = None;
			if !has_loop_exit {
				for pred in preds.iter() {
					let pred = pred.borrow();
					if in_var.is_none() {
						in_var = phi.get_incoming_value_for_block(&pred.label());
					} else if in_var != phi.get_incoming_value_for_block(&pred.label()) {
						in_var = None;
						break;
					}
				}
			}
			if let Some(v) = in_var {
				// If all incoming values for the new PHI would be the same, just don't
				// make a new PHI.  Instead, just remove the incoming values from the old
				// PHI.
				phi
					.source
					.retain(|(_, l)| !preds.iter().any(|b| b.borrow().label() == *l));
				phi.source.push((v, new_bb.borrow().label()));
				self
					.opter
					.temp_graph
					.temp_to_instr
					.get_mut(&phi.target)
					.unwrap()
					.instr = Box::new(phi.clone());
				continue;
			}
			// If the values coming into the block are not the same, we need a new
			// PHI.
			let new_target = self.opter.temp_mgr.new_temp(phi.var_type, false);
			let new_source = phi
				.source
				.iter()
				.filter(|(_, l)| preds.iter().any(|b| b.borrow().label() == *l))
				.cloned()
				.collect::<Vec<(Value, Label)>>();
			phi
				.source
				.retain(|(_, l)| !preds.iter().any(|b| b.borrow().label() == *l));
			phi
				.source
				.push((Value::Temp(new_target.clone()), new_bb.borrow().label()));
			self.opter.temp_graph.temp_to_instr.get_mut(&phi.target).unwrap().instr =
				Box::new(phi.clone());

			let new_phi = PhiInstr::new(new_target.clone(), new_source);
			self
				.opter
				.temp_graph
				.add_temp(new_target.clone(), Box::new(new_phi.clone()));
			new_bb.borrow_mut().phi_instrs.push(new_phi);
			self.opter.def_map.insert(new_target, new_bb.clone());
		}
	}
	/// InsertPreheaderForLoop - Once we discover that a loop doesn't have a
	/// preheader, this method is called to insert one.
	fn insert_preheader_for_loop(&mut self, loop_: LoopPtr) -> LlvmNode {
		let loop_brw = loop_.borrow();
		let header_rc = loop_brw.header.clone();
		let mut outside_blocks = Vec::new();
		let loop_blocks = loop_brw
			.blocks_without_subloops(&self.opter.func.cfg, &self.opter.loop_map);
		for prev in header_rc.clone().borrow().prev.iter() {
			if !loop_blocks.contains(prev) {
				outside_blocks.push(prev.clone());
			}
		}
		assert!(!outside_blocks.is_empty());
		let new_bb =
			self.split_block_predecessors(header_rc, outside_blocks, false);
		println!(
			"LoopSimplify: inserted preheader block {}",
			new_bb.borrow().label()
		);
		if let Some(o) = loop_brw.outer.clone() {
			self.opter.loop_map.insert(new_bb.borrow().id, o.upgrade().unwrap());
		}
		new_bb
	}
	fn form_dedicated_exit_blocks(&mut self, loop_: LoopPtr) -> bool {
		let mut flag = false;
		let loop_blocks = loop_
			.borrow()
			.blocks_without_subloops(&self.opter.func.cfg, &self.opter.loop_map);

		let mut rewrite_exit = |exit: LlvmNode| {
			let mut is_dedicated_exit = true;
			let mut in_loop_prev = Vec::new();
			for prev in exit.borrow().prev.iter() {
				if loop_blocks.contains(prev) {
					in_loop_prev.push(prev.clone());
				} else {
					is_dedicated_exit = false;
				}
			}
			assert!(!in_loop_prev.is_empty());

			if is_dedicated_exit {
				return;
			}

			let new_bb = self.split_block_predecessors(exit, in_loop_prev, true);
			println!(
				"LoopSimplify: inserted dedicated exit block {}",
				new_bb.borrow().label()
			);
			if let Some(o) = loop_.borrow().outer.clone() {
				self.opter.loop_map.insert(new_bb.borrow().id, o.upgrade().unwrap());
			}

			flag = true;
		};

		let mut visited = HashSet::new();
		for bb in loop_blocks.iter() {
			for succ in bb.borrow().succ.iter() {
				if !loop_blocks.contains(succ) && visited.insert(succ.borrow().id) {
					rewrite_exit(succ.clone());
				}
			}
		}
		flag
	}
	fn insert_unique_backedge_block(
		&mut self,
		loop_: LoopPtr,
		preheader: LlvmNode,
	) -> Option<LlvmNode> {
		let mut backedge_blocks = Vec::new();
		let header = loop_.borrow().header.clone();
		for prev in header.borrow().prev.iter() {
			if *prev != preheader {
				backedge_blocks.push(prev.clone());
			}
		}
		if backedge_blocks.len() <= 1 {
			return None;
		}

		let new_bb = self.split_block_predecessors(header, backedge_blocks, false);
		println!(
			"LoopSimplify: inserted unique backedge block {}",
			new_bb.borrow().label()
		);
		self.opter.loop_map.insert(new_bb.borrow().id, loop_.clone());
		Some(new_bb)
	}
	fn simplify_one_loop(&mut self, loop_: LoopPtr) -> bool {
		let mut flag = false;
		// Check to see that no blocks (other than the header) in this loop have
		// predecessors that are not in the loop.  This is not valid for natural
		// loops, but can occur if the blocks are unreachable.
		// 子循环的前驱不可能在本循环外，所以这里可以不遍历子循环的 block
		let blocks_without_subloops = loop_
			.borrow()
			.blocks_without_subloops(&self.opter.func.cfg, &self.opter.loop_map);
		for bb in blocks_without_subloops.iter() {
			let bb = bb.borrow();
			if bb.id == loop_.borrow().header.borrow().id {
				continue;
			}
			// 循环内基本块的前驱不可能在子循环内，否则要么该块属于子循环，要么该块是子循环除 header 以外的入口，而我们的循环都是单一入口的
			for pred in bb.prev.iter() {
				let l = self.opter.loop_map.get(&pred.borrow().id);
				if !l.is_some_and(|l| loop_.borrow().is_super_loop_of(l)) {
					panic!("LoopSimplify: Loop contains a block with a predecessor that is not in the loop!");
				}
			}
		}
		// Does the loop already have a preheader?  If not, insert one.
		let preheader = loop_
			.borrow()
			.get_loop_preheader(&blocks_without_subloops)
			.unwrap_or_else(|| {
				flag = true;
				self.insert_preheader_for_loop(loop_.clone())
			});

		// Next, check to make sure that all exit nodes of the loop only have
		// predecessors that are inside of the loop.  This check guarantees that the
		// loop preheader/header will dominate the exit blocks.  If the exit block has
		// predecessors from outside of the loop, split the edge now.
		flag |= self.form_dedicated_exit_blocks(loop_.clone());

		// If the header has more than two predecessors at this point (from the
		// preheader and from multiple backedges), we must adjust the loop.
		// We do not have nested loops sharing one header, so insert a new block that all backedges target, then make it jump to the loop header.
		flag |=
			self.insert_unique_backedge_block(loop_.clone(), preheader).is_some();
		// If this loop has multiple exits and the exits all go to the same
		// block, attempt to merge the exits. This helps several passes, such
		// as LoopRotation, which do not support loops with multiple exits.
		// SimplifyCFG also does this (and this code uses the same utility
		// function), however this code is loop-aware, where SimplifyCFG is
		// not. That gives it the advantage of being able to hoist
		// loop-invariant instructions out of the way to open up more
		// opportunities, and the disadvantage of having the responsibility
		// to preserve dominator information.
		// TODO: 这一步有点复杂，而且感觉并不能带来很多优化
		flag
	}

	fn simplify_header_phis(
		&self,
		loop_: LoopPtr,
		replace_map: &mut HashMap<LlvmTemp, Value>,
	) {
		let loop_ = loop_.borrow_mut();
		let mut header = loop_.header.borrow_mut();
		// 逆向遍历，下标遍历，一边遍历一边删除
		for phi_idx in (0..header.phi_instrs.len()).rev() {
			let target = header.phi_instrs[phi_idx].target.clone();
			let source = header.phi_instrs[phi_idx].source.clone();
			if source.len() != 2 {
				println!("LoopSimplify: Failed to insert preheader or unique backage");
				break;
			}
			if source[0].0.unwrap_temp().is_some_and(|t| t == target) {
				replace_map.insert(target, source[1].0.clone());
				header.phi_instrs.remove(phi_idx);
			} else if source[1].0.unwrap_temp().is_some_and(|t| t == target) {
				replace_map.insert(target, source[0].0.clone());
				header.phi_instrs.remove(phi_idx);
			}
		}
	}
}
