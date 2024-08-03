use std::{cell::RefCell, fmt::Display, rc::Rc};

use super::LlvmNode;

// use self::utils::is_legal_to_hoist_into;

pub type LoopPtr = Rc<RefCell<Loop>>;

pub mod loop_analysis;
pub mod loop_info;

// Instances of this class are used to represent loops that are detected in the flow graph.
#[derive(Clone, PartialEq, Eq)]
pub struct Loop {
	// 外层 loop
	pub outer: Option<LoopPtr>,
	// 循环头，即 loop 的入口
	pub header: LlvmNode,
	// 循环的嵌套层数
	pub level: i32,
	// 子 loop
	pub subloops: Vec<LoopPtr>,
	// loop 中的所有 block，不包括子 loop 中的 block
	pub blocks: Vec<LlvmNode>,
}

#[allow(unused)]
impl Loop {
	fn new(header: LlvmNode) -> Self {
		Self {
			outer: None,
			header,
			level: -1,
			subloops: Vec::new(),
			blocks: Vec::new(),
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
	pub fn get_loop_preheader(&self) -> Option<LlvmNode> {
		let pred = self.get_loop_predecessor()?;
		if !is_legal_to_hoist_into(pred.clone()) {
			println!("Preheader is not legal to hoist into");
			return None;
		}
		if pred.borrow().succ.len() != 1 || pred.borrow().succ[0] != self.header {
			println!("Multiple preheaders or Illiagal preheader");
			return None;
		}
		println!("Found a preheader {}", pred.borrow().label());
		Some(pred)
	}
	/// getLoopPredecessor - If the given loop's header has exactly one unique
	/// predecessor outside the loop, return it. Otherwise return None.
	/// This is less strict that the loop "preheader" concept, which requires
	/// the predecessor to have exactly one successor.
	///
	pub fn get_loop_predecessor(&self) -> Option<LlvmNode> {
		let header = self.header.borrow();
		let mut pred = None;
		for pred_ in header.prev.iter() {
			if !self.contains_block(pred_) {
				if pred.is_some() && pred != Some(pred_.clone()) {
					return None;
				}
				pred = Some(pred_.clone());
			}
		}
		println!(
			"Found a predecessor {}",
			pred.as_ref().unwrap().borrow().label()
		);
		pred
	}
	/// getLoopLatch - If there is a single latch block for this loop, return it.
	/// A latch block is a block that contains a branch back to the header.
	pub fn get_loop_latch(&self) -> Option<LlvmNode> {
		let header = self.header.borrow();
		let mut latch = None;
		for pred in header.prev.iter() {
			if self.contains_block(pred) {
				if latch.is_some() {
					return None;
				}
				latch = Some(pred.clone());
			}
		}
		latch
	}
	// 不仅看自己包不包含该 block，还要看自己的子 loop 有没有包含该 block
	fn contains_block(&self, block: &LlvmNode) -> bool {
		self.blocks.contains(block)
			|| self.subloops.iter().any(|loop_| loop_.borrow().contains_block(block))
	}
	fn no_inner(&self) -> bool {
		self.subloops.is_empty()
	}
}

impl Display for Loop {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let outer = if let Some(outer) = &self.outer {
			format!("outer: {}", outer.borrow().header.borrow().id)
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