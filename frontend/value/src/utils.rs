use llvm::llvmvar::VarType;

use crate::Value;

impl Value {
	pub fn get_type(&self) -> VarType {
		match &self {
			Self::Int(_) => VarType::I32,
			Self::Float(_) => VarType::F32,
			Self::IntPtr(_, _) => VarType::I32Ptr,
			Self::FloatPtr(_, _) => VarType::F32Ptr,
		}
	}
}
