use std::collections::HashMap;

#[derive(Default)]
pub struct UnionFind {
	fa: HashMap<i32, i32>,
}

impl UnionFind {
	pub fn find(&mut self, x: i32) -> i32 {
		if let Some(v) = self.fa.get(&x).copied() {
			let out = self.find(v);
			self.fa.insert(x, v);
			out
		} else {
			x
		}
	}
	pub fn merge(&mut self, x: i32, y: i32) {
		let x = self.find(x);
		self.fa.insert(x, y);
	}
	pub fn same(&mut self, x: i32, y: i32) -> bool {
		self.find(x) == self.find(y)
	}
}
