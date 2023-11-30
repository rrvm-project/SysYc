use std::collections::HashMap;

use llvm::Value;

pub type Table = HashMap<i32, Value>;

pub struct SymbolTable {
	pub stack: Vec<Table>,
}

impl SymbolTable {
	pub fn new() -> Self {
		Self { stack: Vec::new() }
	}
	pub fn push(&mut self) {
		self.stack.push(HashMap::new())
	}
	// TODO: use persistent data structures to make it faster
	pub fn get_skip(&self, id: &i32, offset: usize) -> Option<Value> {
		for table in self.stack.iter().rev().skip(offset) {
			if let Some(temp) = table.get(id) {
				return Some(temp.clone());
			}
		}
		None
	}
	pub fn top(&self, n: usize) -> Table {
		let mut out = HashMap::new();
		for table in self.stack.iter().rev().take(n) {
			for (k, v) in table.iter() {
				out.entry(*k).or_insert_with(|| v.clone());
			}
		}
		out
	}
	pub fn get(&self, id: &i32) -> Value {
		self.get_skip(id, 0).unwrap()
	}
	pub fn set(&mut self, id: i32, temp: Value) {
		if let Some(last) = self.stack.last_mut() {
			last.insert(id, temp);
		}
	}
	pub fn pop(&mut self) {
		let top = self.stack.pop().unwrap();
		if let Some(last) = self.stack.last_mut() {
			for (k, v) in top.into_iter() {
				last.insert(k, v);
			}
		}
	}
	pub fn drop(&mut self) -> Table {
		self.stack.pop().unwrap()
	}
}

impl Default for SymbolTable {
	fn default() -> Self {
		Self::new()
	}
}
