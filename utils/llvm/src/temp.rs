use std::fmt::Display;

use utils::TempTrait;

use crate::{llvmop::Value, llvmvar::VarType};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Temp {
	pub name: String,
	pub is_global: bool,
	pub var_type: VarType,
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

impl TempTrait for Temp {}

impl Temp {
	fn new(name: impl Display, var_type: VarType, is_global: bool) -> Self {
		Self {
			name: name.to_string(),
			var_type,
			is_global,
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
	pub total: u32,
}

impl TempManager {
	pub fn new() -> Self {
		Self::default()
	}
	pub fn new_temp(&mut self, var_type: VarType, is_global: bool) -> Temp {
		self.total += 1;
		Temp::new(self.total, var_type, is_global)
	}
	pub fn new_temp_with_name(
		&mut self,
		name: String,
		var_type: VarType,
	) -> Temp {
		self.total += 1;
		Temp::new(name, var_type, true)
	}
}
