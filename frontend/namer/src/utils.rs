use ir_type::builtin_type::{BaseType, IRType};

use utils::InitValueItem;

use scope::symbol::{FuncSymbol, VarSymbol};
use std::{collections::HashMap, vec};
use utils::SysycError;

pub fn assert_is_convertable_to(
	this: &IRType,
	other: &IRType,
) -> Result<(), SysycError> {
	if this.is_const && !other.is_const {
		return Err(SysycError::SyntaxError(
			"can not convert const into nonconst".to_string(),
		));
	}

	if this.is_array() != other.is_array() {
		return Err(SysycError::SyntaxError(
			"can not convert between scalar and array".to_string(),
		));
	}
	if this.base_type == BaseType::Void || other.base_type == BaseType::Void {
		return Err(SysycError::SyntaxError(
			"can not convert Void type".to_string(),
		));
	}
	if this.dim_length() != other.dim_length() {
		return Err(SysycError::SyntaxError(
			"can not convert between arrays of different dims".to_string(),
		));
	}

	for i in 1..this.dim_length() {
		if this.dims[i] != other.dims[i] {
			return  Err(SysycError::SyntaxError(format!("can not convert between arrays of different size. At dim {}, try convert {} to {}",i, this.dims[i], other.dims[i])));
		}
	}
	Ok(())
}

pub fn array_init_for_backend<T: std::fmt::Debug>(
	source: &HashMap<usize, T>,
	trans: fn(&T) -> InitValueItem,
) -> Vec<InitValueItem> {
	let mut parts: Vec<_> = source.iter().collect();
	let mut result = vec![];

	parts.sort_by_key(|&(index, _)| index);

	let mut last: usize = 0;

	for (index, value) in parts.iter() {
		if **index > last {
			result.push(InitValueItem::None(**index - last));
			last = **index;
		}
		result.push(trans(value));
		last += 1;
	}
	result
}

#[derive(Debug)]
pub struct DataFromNamer {
	pub global_var_init_value: HashMap<String, Vec<InitValueItem>>,
	pub var_symbols: Vec<VarSymbol>,
	pub func_symbols: Vec<FuncSymbol>,
}
