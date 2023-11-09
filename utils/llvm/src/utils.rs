use crate::{llvmop::Value, llvmvar::VarType, temp::Temp};

pub fn all_equal<T: PartialEq>(slice: &[T]) -> bool {
	slice.windows(2).all(|window| window[0] == window[1])
}

pub fn unwrap_values(arr: Vec<&Value>) -> Vec<Temp> {
	arr.into_iter().flat_map(|v| v.unwrap_temp()).collect()
}

pub fn type2ptr(var_type: VarType) -> VarType {
	match var_type {
		VarType::F32 => VarType::F32Ptr,
		VarType::I32 => VarType::I32Ptr,
		_ => unreachable!(),
	}
}

pub fn ptr2type(var_type: VarType) -> VarType {
	match var_type {
		VarType::F32Ptr => VarType::F32,
		VarType::I32Ptr => VarType::I32,
		_ => unreachable!(),
	}
}

pub fn is_ptr(var_type: VarType) -> bool {
	match var_type {
		VarType::F32Ptr => true,
		VarType::I32Ptr => true,
		_ => false,
	}
}

pub fn type_match_ptr(var_type: VarType, ptr_type: VarType) -> bool {
	ptr_type == type2ptr(var_type)
}
