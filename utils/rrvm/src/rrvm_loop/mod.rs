use std::{cell::RefCell, fmt::Display, rc::Rc};

use crate::LlvmNode;

pub type LoopPtr = Rc<RefCell<Loop>>;

pub mod loop_analysis;
pub mod loop_info;

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
