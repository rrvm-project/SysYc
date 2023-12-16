use std::fmt::Display;

use crate::{llvmop::Value, llvmvar::VarType};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Temp {
	pub name: String,
	pub var_type: VarType,
}

impl Display for Temp {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "%{}", self.name)
	}
}

impl Temp {
	fn new(id: u32, var_type: VarType) -> Self {
		Self {
			name: id.to_string(),
			var_type,
		}
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
}
