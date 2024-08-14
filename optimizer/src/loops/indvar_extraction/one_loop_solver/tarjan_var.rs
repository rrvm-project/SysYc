use std::collections::{HashMap, HashSet};

use llvm::LlvmTemp;

pub struct TarjanVar {
	// dfs 过程中，访问到的次序
	pub dfsnum: HashMap<LlvmTemp, i32>,
	pub next_dfsnum: i32,
	pub visited: HashSet<LlvmTemp>,
	// Tarjan 算法计算强连通分量时，需要用到的值
	pub low: HashMap<LlvmTemp, i32>,
	pub stack: Vec<LlvmTemp>,
	pub in_stack: HashSet<LlvmTemp>,
}

impl TarjanVar {
	pub fn new() -> Self {
		Self {
			dfsnum: HashMap::new(),
			next_dfsnum: 0,
			visited: HashSet::new(),
			low: HashMap::new(),
			stack: Vec::new(),
			in_stack: HashSet::new(),
		}
	}
}
