use std::fmt::Display;

use utils::TempTrait;

use crate::llvmvar::VarType;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LlvmTemp {
	pub name: String,
	pub is_global: bool,
	pub var_type: VarType,
}

impl std::fmt::Debug for LlvmTemp {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		// f.debug_struct("LlvmTemp").field("name", &self.name).field("is_global", &self.is_global).field("var_type", &self.var_type).finish()
		let mut expression = vec![];
		if self.is_global {
			expression.push("G");
		}
		expression.push("%");
		expression.push(match self.var_type {
			VarType::I32 => "I",
			VarType::F32 => "F",
			VarType::I32Ptr => "I*",
			VarType::F32Ptr => "F*",
			VarType::Void => "V",
		});
		expression.push(&self.name);
		f.write_str(expression.join("").as_str())
	}
}

impl Display for LlvmTemp {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		if self.is_global {
			write!(f, "@{}", self.name)
		} else {
			write!(f, "%{}", self.name)
		}
	}
}

impl TempTrait for LlvmTemp {}

impl LlvmTemp {
	pub fn new(name: impl Display, var_type: VarType, is_global: bool) -> Self {
		Self {
			name: name.to_string(),
			var_type,
			is_global,
		}
	}
}

#[derive(Default)]
pub struct LlvmTempManager {
	pub total: u32,
}

impl LlvmTempManager {
	pub fn new() -> Self {
		Self::default()
	}
	pub fn new_temp(&mut self, var_type: VarType, is_global: bool) -> LlvmTemp {
		self.total += 1;
		LlvmTemp::new(self.total, var_type, is_global)
	}
	pub fn new_temp_with_name(
		&mut self,
		name: String,
		var_type: VarType,
	) -> LlvmTemp {
		self.total += 1;
		LlvmTemp::new(name, var_type, true)
	}
}
