use crate::{BType, Value};

impl Value {
	pub fn get_type(&self) -> BType {
		match &self {
			Self::Int(_) => BType::Int,
			Self::Float(_) => BType::Float,
			_ => unreachable!(),
		}
	}
}
