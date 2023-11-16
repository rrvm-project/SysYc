use crate::{FloatPtr, IntPtr, Value};
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
