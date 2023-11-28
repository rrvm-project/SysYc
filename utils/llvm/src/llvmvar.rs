use std::fmt::Display;

use crate::Value;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum VarType {
	I32,
	F32,
	I32Ptr,
	F32Ptr,
	Void,
}

impl Display for VarType {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let type_str = match self {
			Self::I32 => "i32",
			Self::I32Ptr => "i32*",
			Self::F32 => "f32",
			Self::F32Ptr => "f32*",
			Self::Void => "void",
		};
		write!(f, "{}", type_str)
	}
}

impl VarType {
	pub fn default_value(&self) -> Value {
		match self {
			Self::I32 => 0.into(),
			Self::F32 => 0.0.into(),
			_ => unreachable!(),
		}
	}
}
