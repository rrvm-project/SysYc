use llvm::llvmvar::VarType;

use crate::Value;

use utils::errors::Result;

impl Value {
	pub fn get_type(&self) -> VarType {
		match &self {
			Self::Int(_) => VarType::I32,
			Self::Float(_) => VarType::F32,
			Self::IntPtr(_) => VarType::I32Ptr,
			Self::FloatPtr(_) => VarType::F32Ptr,
		}
	}

	pub fn get_i32_value(&self) -> Result<i32> {
		match &self {
			Self::Int(v) => Ok(*v),
			Self::Float(v) => Ok(*v as i32),
			_ => Err(utils::SysycError::SyntaxError(
				"prt can not be converted into i32".to_string(),
			)),
		}
	}

	pub fn get_f32_value(&self) -> Result<f32> {
		match &self {
			Self::Int(v) => Ok(*v as f32),
			Self::Float(v) => Ok(*v),
			_ => Err(utils::SysycError::SyntaxError(
				"prt can not be converted into f32".to_string(),
			)),
		}
	}
}
