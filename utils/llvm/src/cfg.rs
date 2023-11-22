use std::{collections::HashMap, fmt::Display};

use crate::basicblock::BasicBlock;

#[allow(unused)]
pub struct CFG {
	// id 到 basicblock 的映射
	pub basic_blocks: HashMap<usize, BasicBlock>,
	pub entry: usize,
	pub exit: usize,
}

impl CFG {
	pub fn new(entry: BasicBlock, exit: BasicBlock) -> CFG {
		CFG {
			entry: entry.id,
			basic_blocks: {
				let mut map = HashMap::new();
				map.insert(entry.id, entry);
				map.insert(exit.id, exit);
				map
			},
			exit: 0,
		}
	}
}

impl Display for CFG {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		for (id, bb) in &self.basic_blocks {
			if *id == 1 {
				continue;
			}
			writeln!(f, "{}", bb)?;
		}
		write!(f, "{}", self.basic_blocks.get(&1).unwrap())?;
		Ok(())
	}
}
