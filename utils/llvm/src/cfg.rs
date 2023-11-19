use std::fmt::Display;

use crate::basicblock::BasicBlock;

#[allow(unused)]
pub struct CFG {
	pub basic_blocks: Vec<BasicBlock>,
	pub entry: usize,
	pub exit: usize,
}

impl CFG {
	pub fn new(entry: BasicBlock, exit: BasicBlock) -> CFG {
		CFG {
			entry: entry.id,
			basic_blocks: vec![entry, exit],
			exit: 0,
		}
	}

	pub fn set_exit(&mut self, exit: BasicBlock) {
		self.exit = exit.id;
		self.basic_blocks.push(exit);
	}
}

impl Display for CFG {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		for bb in &self.basic_blocks {
			if bb.id == 1 {
				continue;
			}
			writeln!(f, "{}", bb)?;
		}
		write!(f, "{}", self.basic_blocks[1])?;
		Ok(())
	}
}
