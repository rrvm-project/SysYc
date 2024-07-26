use std::collections::{HashMap, HashSet};

use super::indvar::IndVar;
use llvm::LlvmTemp;

#[allow(unused)]
pub struct LoopInfo {
	indvars: HashMap<LlvmTemp, IndVar>,
	// 大多循环变量会组成一个胖链条，以一个在循环入口处的 phi 语句为 header
	chain_header: HashMap<LlvmTemp, LlvmTemp>,
	// 循环不变量
	invariant: HashSet<LlvmTemp>,
	// 循环变量
	variant: HashSet<LlvmTemp>,
}

impl LoopInfo {
	pub fn new() -> LoopInfo {
		Self {
			indvars: HashMap::new(),
			chain_header: HashMap::new(),
			invariant: HashSet::new(),
			variant: HashSet::new(),
		}
	}
}
