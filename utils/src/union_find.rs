use core::hash::Hash;
use std::collections::HashMap;

pub struct UnionFind<T: Hash + Eq + Copy> {
	fa: HashMap<T, T>,
}

impl<T: Hash + Eq + Copy> UnionFind<T> {
	pub fn find(&mut self, x: T) -> T {
		if let Some(v) = self.fa.get(&x).copied() {
			let out = self.find(v);
			self.fa.insert(x, v);
			out
		} else {
			x
		}
	}
	pub fn merge(&mut self, x: T, y: T) {
		let x = self.find(x);
		self.fa.insert(x, y);
	}
	pub fn same(&mut self, x: T, y: T) -> bool {
		self.find(x) == self.find(y)
	}
	pub fn is_root(&mut self, x: T) -> bool {
		x == self.find(x)
	}
}

impl<T: Hash + Eq + Copy> Default for UnionFind<T> {
	fn default() -> Self {
		Self { fa: HashMap::new() }
	}
}
