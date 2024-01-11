use std::{cell::RefCell, rc::Rc};

use crate::LlvmNode;

pub type LoopPtr = Rc<RefCell<Loop>>;

pub mod find_loop;

#[allow(unused)]
#[derive(Clone, PartialEq, Eq)]
pub struct Loop {
	pub outer: Option<LoopPtr>,
	pub header: LlvmNode,
	pub level: i32,
	pub no_inner: bool,
}

#[allow(unused)]
impl Loop {
	fn new(header: LlvmNode) -> Self {
		Self {
			outer: None,
			header,
			level: -1,
			no_inner: true,
		}
	}
}
