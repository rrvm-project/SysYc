use crate::{Array, Value};

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

impl From<(Vec<usize>, Array<i32>)> for Value {
	fn from(value: (Vec<usize>, Array<i32>)) -> Self {
		let (index, array) = value;
		Value::IntPtr(index, array)
	}
}

impl From<(Vec<usize>, Array<f32>)> for Value {
	fn from(value: (Vec<usize>, Array<f32>)) -> Self {
		let (index, array) = value;
		Value::FloatPtr(index, array)
	}
}
