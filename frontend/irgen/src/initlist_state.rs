use llvm::{Value, VarType};

use crate::visitor::Item;

pub struct InitlistState {
	pub var_type: VarType,
	pub decl_dims: Vec<usize>,
	pub depth: usize,
	pub values: Vec<Vec<Item>>,
}

impl InitlistState {
	pub fn new(var_type: VarType, decl_dims: Vec<usize>) -> Self {
		Self {
			var_type,
			decl_dims,
			depth: 0,
			values: Vec::new(),
		}
	}
	pub fn cur_size(&self) -> usize {
		self.decl_dims.iter().skip(self.depth).product()
	}
	pub fn push(&mut self) {
		self.depth += 1;
		self.values.push(Vec::new());
	}
	pub fn pop(&mut self) -> Vec<Item> {
		self.depth -= 1;
		self.values.pop().unwrap()
	}
	pub fn store(&mut self, item: Item) {
		self.values.last_mut().unwrap().push(item)
	}
	pub fn default_init_val(&mut self) -> Value {
		self.var_type.default_value()
	}
	pub fn top_len(&self) -> usize {
		self.values.last().unwrap().len()
	}
}
