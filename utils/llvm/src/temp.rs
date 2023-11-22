use std::fmt::Display;

use crate::{llvmop::Value, llvmvar::VarType};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Temp {
	pub name: String,
	pub var_type: VarType,
	pub is_global: bool,
}

impl Display for Temp {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		if self.is_global {
			write!(f, "@{}", self.name)
		} else {
			write!(f, "%{}", self.name)
		}
	}
}

impl Temp {
	pub fn new(id: u32, var_type: VarType) -> Self {
		Self {
			name: id.to_string(),
			var_type,
			is_global: false,
		}
	}
	pub fn new_global(name: String, var_type: VarType) -> Self {
		Self {
			name,
			var_type,
			is_global: true,
		}
	}
	pub fn is_ptr(&self) -> bool {
		self.var_type == VarType::I32Ptr || self.var_type == VarType::F32Ptr
	}
}

impl Value {
	pub fn unwrap_temp(&self) -> Option<Temp> {
		match self {
			Self::Temp(v) => Some(v.clone()),
			_ => None,
		}
	}
}

#[derive(Default)]
pub struct TempManager {
	total: u32,
}

impl TempManager {
	pub fn new() -> Self {
		Self::default()
	}
	pub fn new_temp(&mut self, var_type: VarType) -> Temp {
		self.total += 1;
		Temp::new(self.total, var_type)
	}
	pub fn cur_total(&self) -> u32 {
		self.total
	}
	pub fn set_total(&mut self, total: u32) {
		self.total = total;
	}
}
