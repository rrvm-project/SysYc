use std::{
	cell::RefCell,
	collections::{HashMap, HashSet},
	fmt::Display,
	hash::Hash,
	rc::{Rc, Weak},
};

use crate::LlvmCFG;

use super::LlvmNode;

pub type LoopPtr = Rc<RefCell<Loop>>;
pub type LoopWeakPtr = Weak<RefCell<Loop>>;

pub mod loop_analysis;

// Instances of this class are used to represent loops that are detected in the flow graph.
#[derive(Clone)]
pub struct Loop {
	//
	pub id: u32,     // dfs的begin
	pub ura_id: u32, // 里id, dfs的end
	// 外层 loop
	// 防止内存泄漏！
	pub outer: Option<LoopWeakPtr>,
	// 循环头，即 loop 的入口
	pub header: LlvmNode,
	// 循环的嵌套层数, 一层循环为 1, 二层循环为 2, 被视为一个只执行一次的循环的整个控制流为 0
	pub level: i32,
	// 子 loop
	pub subloops: Vec<LoopPtr>,
	// loop 中的所有 block，不包括子 loop 中的 block
}

#[allow(unused)]
impl Loop {
	pub fn is_strict_super_loop_of(&self, other: &LoopPtr) -> bool {
		let other = other.borrow();
		self.id < other.id && other.ura_id < self.ura_id
	}

	pub fn is_super_loop_of(&self, other: &LoopPtr) -> bool {
		let other = other.borrow();
		(self.id < other.id && other.ura_id < self.ura_id)
			|| (self.id == other.id && self.ura_id == other.ura_id)
	}

	fn new(header: LlvmNode) -> Self {
		Self {
			id: 0,
			ura_id: 0,
			outer: None,
			header,
			level: -1,
			subloops: Vec::new(),
		}
	}
	/// getLoopPreheader - If there is a preheader for this loop, return it.  A
	/// loop has a preheader if there is only one edge to the header of the loop
	/// from outside of the loop and it is legal to hoist instructions into the
	/// predecessor. If this is the case, the block branching to the header of the
	/// loop is the preheader node.
	///
	/// This method returns null if there is no preheader for the loop.
	///
	/// @param blocks - The set of blocks in the loop, not containing blocks in subloops.
	pub fn get_loop_preheader(&self, blocks: &[LlvmNode]) -> Option<LlvmNode> {
		let pred = self.get_loop_predecessor(blocks)?;
		if !is_legal_to_hoist_into(pred.clone()) {
			println!("Preheader is not legal to hoist into");
			return None;
		}
		if pred.borrow().succ.len() != 1 || pred.borrow().succ[0] != self.header {
			println!("Multiple preheaders or Illiagal preheader");
			return None;
		}
		Some(pred)
	}
	/// getLoopPredecessor - If the given loop's header has exactly one unique
	/// predecessor outside the loop, return it. Otherwise return None.
	/// This is less strict that the loop "preheader" concept, which requires
	/// the predecessor to have exactly one successor.
	/// @param blocks - The set of blocks in the loop, not containing blocks in subloops.
	pub fn get_loop_predecessor(&self, blocks: &[LlvmNode]) -> Option<LlvmNode> {
		let header = self.header.borrow();
		let mut pred = None;
		for pred_ in header.prev.iter() {
			if !blocks.contains(pred_) {
				if pred.is_some_and(|p| p != pred_.clone()) {
					return None;
				}
				pred = Some(pred_.clone());
			}
		}
		pred
	}
	/// getLoopLatch - If there is a single latch block for this loop, return it.
	/// A latch block is a block that contains a branch back to the header.
	/// @param blocks - The set of blocks in the loop, not containing blocks in subloops.
	pub fn get_loop_latch(&self, blocks: &[LlvmNode]) -> Option<LlvmNode> {
		let header = self.header.borrow();
		let mut latch = None;
		for pred in header.prev.iter() {
			if blocks.contains(pred) {
				if latch.is_some() {
					return None;
				}
				latch = Some(pred.clone());
			}
		}
		latch
	}
	fn no_inner(&self) -> bool {
		self.subloops.is_empty()
	}
	// 临时计算 loop 内有哪些 block, 包括子循环的 block
	pub fn blocks(
		&self,
		cfg: &LlvmCFG,
		loop_map: &HashMap<i32, LoopPtr>,
	) -> Vec<LlvmNode> {
		// 从 header 开始，遍历在同一循环内的后继，直到回到 header
		let mut deduplicate = HashSet::new();
		let mut visited = Vec::new();
		let mut stack = vec![self.header.clone()];
		while let Some(bb) = stack.pop() {
			if deduplicate.insert(bb.borrow().id) {
				visited.push(bb.clone());
				for succ in bb.borrow().succ.iter() {
					if loop_map
						.get(&succ.borrow().id)
						.is_some_and(|l| self.is_super_loop_of(l))
					{
						stack.push(succ.clone());
					}
				}
			}
		}
		visited
	}
	// 临时计算 loop 内有哪些 block, 不包括子循环的 block
	pub fn blocks_without_subloops(
		&self,
		cfg: &LlvmCFG,
		loop_map: &HashMap<i32, LoopPtr>,
	) -> Vec<LlvmNode> {
		// 从 header 开始，遍历在同一循环内的后继，直到回到 header
		let mut deduplicate = HashSet::new();
		let mut visited = Vec::new();
		let mut stack = vec![self.header.clone()];
		while let Some(bb) = stack.pop() {
			if deduplicate.insert(bb.borrow().id) {
				visited.push(bb.clone());
				for succ in bb.borrow().succ.iter() {
					if let Some(l) = loop_map.get(&succ.borrow().id) {
						if l.borrow().id == self.id {
							stack.push(succ.clone());
						} else if self.is_super_loop_of(l) {
							stack.push(l.borrow().header.clone());
						}
					}
				}
			}
		}
		visited
	}
}

impl Hash for Loop {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.id.hash(state);
	}
}

impl Display for Loop {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let outer = if let Some(outer) = &self.outer {
			if let Some(outer) = outer.upgrade() {
				format!("outer: {}", outer.borrow().id)
			} else {
				"outer: None".to_string()
			}
		} else {
			"outer: None".to_string()
		};
		write!(
			f,
			"outer: {}, header: {}, level: {}",
			outer,
			self.header.borrow().id,
			self.level,
		)
	}
}

pub fn is_legal_to_hoist_into(bb: LlvmNode) -> bool {
	assert!(!bb.borrow().succ.is_empty());
	return !bb.borrow().jump_instr.as_ref().unwrap().is_ret();
}
