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

impl Default for VarType {
	fn default() -> Self {
		Self::Void
	}
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

pub fn upgrade(x: VarType, y: VarType) -> Option<VarType> {
	if x.is_ptr() || y.is_ptr() {
		None
	} else {
		Some(match (x, y) {
			(VarType::I32, VarType::I32) => VarType::I32,
			(_, VarType::F32) | (VarType::F32, _) => VarType::F32,
			_ => unreachable!(),
		})
	}
}

impl VarType {
	pub fn default_value(&self) -> Value {
		match self {
			Self::F32 => 0.0.into(),
			_ => 0.into(),
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
	pub fn to_ptr(&self) -> VarType {
		match self {
			Self::I32 => Self::I32Ptr,
			Self::F32 => Self::F32Ptr,
			_ => *self,
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
	pub fn is_void(&self) -> bool {
		matches!(self, Self::Void)
	}
	pub fn is_ptr(&self) -> bool {
		matches!(self, Self::F32Ptr | Self::I32Ptr)
	}
	pub fn is_float(&self) -> bool {
		matches!(self, Self::F32)
	}
}
