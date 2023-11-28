use std::collections::HashMap;

use llvm::Value;

pub struct SymbolTable {
	pub stack: Vec<HashMap<i32, Value>>,
}

impl SymbolTable {
	pub fn new() -> Self {
		Self { stack: Vec::new() }
	}
	// set this if is possible to make phi instr
	pub fn push(&mut self) {
		self.stack.push(HashMap::new())
	}
	pub fn get(&mut self, id: &i32) -> Value {
		for table in self.stack.iter().rev() {
			if let Some(temp) = table.get(id) {
				return temp.clone();
			}
		}
		unreachable!()
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
				last.entry(k).and_modify(|val| *val = v);
			}
		}
	}
}
