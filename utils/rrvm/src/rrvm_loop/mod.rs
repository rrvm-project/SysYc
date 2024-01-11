use std::{cell::RefCell, rc::Rc};

use crate::LlvmNode;

pub type LoopPtr = Rc<RefCell<Loop>>;

#[allow(unused)]
pub struct Loop {
	outer: Option<LoopPtr>,
	header: LlvmNode,
	level: i32,
	no_inner: bool,
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
