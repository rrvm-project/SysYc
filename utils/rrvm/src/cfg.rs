use crate::basicblock::Node;

pub struct CFG {
	pub total: i32,
	pub blocks: Vec<Node>, // 锁定 0 是 enter，1 是 exit
}

impl CFG {
	pub fn new() -> Self {
		Self {
			total: 0,
			blocks: Vec::new(),
		}
	}
	pub fn merge(&mut self, mut other: CFG) {
		other.blocks.iter().for_each(|v| v.borrow_mut().id += self.total);
		self.total += other.total;
		self.blocks.append(&mut other.blocks);
	}
}

impl Default for CFG {
	fn default() -> Self {
		Self::new()
	}
}
