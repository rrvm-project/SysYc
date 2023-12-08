use std::fmt::Display;

use crate::{llvmop::ArithOp, Value};

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
	pub fn default_value_option(&self) -> Option<Value> {
		match self {
			Self::I32 => Some(0.into()),
			Self::F32 => Some(0.0.into()),
			Self::Void => None,
			_ => unreachable!(),
		}
	}
	pub fn deref_type(&self) -> VarType {
		match self {
			Self::I32Ptr => Self::I32,
			Self::F32Ptr => Self::F32,
			_ => unreachable!(),
		}
	}
	pub fn move_op(&self) -> ArithOp {
		match self {
			Self::F32 => ArithOp::Fadd,
			Self::Void => unreachable!(),
			_ => ArithOp::Add,
		}
	}
	pub fn get_size(&self) -> usize {
		match self {
			Self::I32 | Self::I32Ptr | Self::F32 | Self::F32Ptr => 4,
			Self::Void => unreachable!(),
		}
	}
}
