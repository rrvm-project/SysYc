mod dominator_frontier;
mod impls;
mod naive;

use std::collections::HashMap;

pub use dominator_frontier::*;
pub use naive::*;

use crate::LlvmNode;

#[derive(Default)]
pub struct DomTree {
	pub dominates: HashMap<i32, Vec<LlvmNode>>,
	pub dominator: HashMap<i32, LlvmNode>,
	pub dom_direct: HashMap<i32, Vec<LlvmNode>>,
	pub df: HashMap<i32, Vec<LlvmNode>>,
}
