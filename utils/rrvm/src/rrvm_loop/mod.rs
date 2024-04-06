use std::{cell::RefCell, fmt::Display, rc::Rc};

use log::trace;

use crate::LlvmNode;

use self::utils::is_legal_to_hoist_into;

pub type LoopPtr = Rc<RefCell<Loop>>;

pub mod loop_analysis;
pub mod loop_info;
pub mod utils;

// Instances of this class are used to represent loops that are detected in the flow graph.
#[derive(Clone, PartialEq, Eq)]
pub struct Loop {
	pub outer: Option<LoopPtr>,
	pub header: LlvmNode,
	pub level: i32,
	pub no_inner: bool,
	pub subloops: Vec<LoopPtr>,
	pub blocks: Vec<LlvmNode>,
}

#[allow(unused)]
impl Loop {
	fn new(header: LlvmNode) -> Self {
		Self {
			outer: None,
			header,
			level: -1,
			no_inner: true,
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
			trace!("Preheader is not legal to hoist into");
			return None;
		}
		if pred.borrow().succ.len() != 1 || pred.borrow().succ[0] != self.header {
			trace!("Multiple preheaders or Illiagal preheader");
			return None;
		}
		trace!("Found a preheader {}", pred.borrow().label());
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
				if pred.is_some() && pred != Some(pred_.clone()){
					return None;
				}
				pred = Some(pred_.clone());
			}
		}
		trace!("Found a predecessor {}", pred.as_ref().unwrap().borrow().label());
		pred
	}
	fn contains_block(&self, block: &LlvmNode) -> bool {
		self.blocks.contains(block)
			|| self.subloops.iter().any(|loop_| loop_.borrow().contains_block(block))
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
			"outer: {}, header: {}, level: {}, no_inner: {}",
			outer,
			self.header.borrow().id,
			self.level,
			self.no_inner
		)
	}
}
