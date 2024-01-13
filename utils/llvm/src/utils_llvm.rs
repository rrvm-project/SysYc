use crate::{llvmop::Value, llvmvar::VarType, LlvmTemp};

pub fn all_equal<T: PartialEq>(slice: &[T]) -> bool {
	slice.windows(2).all(|window| window[0] == window[1])
}

pub fn unwrap_values(arr: Vec<&Value>) -> Vec<LlvmTemp> {
	arr.into_iter().flat_map(|v| v.unwrap_temp()).collect()
}

pub fn type2ptr(var_type: VarType) -> VarType {
	match var_type {
		VarType::F32 => VarType::F32Ptr,
		VarType::I32 => VarType::I32Ptr,
		_ => unreachable!(),
	}
}

pub fn is_ptr(var_type: VarType) -> bool {
	matches!(var_type, VarType::F32Ptr | VarType::I32Ptr)
}

pub fn type_match_ptr(var_type: VarType, ptr_type: VarType) -> bool {
	ptr_type == type2ptr(var_type)
}
