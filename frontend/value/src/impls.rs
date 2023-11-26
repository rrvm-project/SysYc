use std::fmt::Display;

use crate::{BType, FloatPtr, FuncRetType, IntPtr, Value, VarType};
use utils::{errors::Result, SysycError::TypeError};

impl From<i32> for Value {
	fn from(value: i32) -> Self {
		Value::Int(value)
	}
}

impl From<f32> for Value {
	fn from(value: f32) -> Self {
		Value::Float(value)
	}
}

impl From<IntPtr> for Value {
	fn from(value: IntPtr) -> Self {
		Value::IntPtr(value)
	}
}

impl From<FloatPtr> for Value {
	fn from(value: FloatPtr) -> Self {
		Value::FloatPtr(value)
	}
}

impl Value {
	pub fn to_int(&self) -> Result<i32> {
		match self {
			Self::Int(v) => Ok(*v),
			Self::Float(v) => Ok(*v as i32),
			_ => Err(TypeError("try to convert pointer to int".to_string())),
		}
	}
	pub fn to_float(&self) -> Result<f32> {
		match self {
			Self::Int(v) => Ok(*v as f32),
			Self::Float(v) => Ok(*v),
			_ => Err(TypeError("try to convert pointer to float".to_string())),
		}
	}
}

impl From<(bool, BType, Vec<usize>)> for VarType {
	fn from(value: (bool, BType, Vec<usize>)) -> Self {
		let (is_lval, type_t, dims) = value;
		Self {
			is_lval,
			type_t,
			dims,
		}
	}
}

impl From<FuncRetType> for Option<VarType> {
	fn from(value: FuncRetType) -> Self {
		match value {
			FuncRetType::Int => Some(VarType::new_int()),
			FuncRetType::Float => Some(VarType::new_float()),
			_ => None,
		}
	}
}

impl VarType {
	pub fn new_int() -> Self {
		Self {
			is_lval: false,
			type_t: BType::Int,
			dims: Vec::new(),
		}
	}
	pub fn new_float() -> Self {
		Self {
			is_lval: false,
			type_t: BType::Float,
			dims: Vec::new(),
		}
	}
}

impl Display for VarType {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		if self.dims.is_empty() {
			write!(f, "{:?}", self.type_t)
		} else {
			let v = self
				.dims
				.iter()
				.skip(1)
				.map(|v| format!("[{}]", v))
				.collect::<Vec<_>>()
				.join("");
			write!(f, "{:?} (*){}", self.type_t, v)
		}
	}
}
