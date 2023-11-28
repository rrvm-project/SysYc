use std::fmt::Display;

use llvm::{Value, VarType};

use crate::cfg::CFG;

pub struct RrvmFunc<T: Display> {
	pub cfg: CFG<T>,
	pub name: String,
	pub ret_type: VarType,
	pub params: Vec<Value>,
}

impl<T: Display> RrvmFunc<T> {
	pub fn new(
		cfg: CFG<T>,
		name: String,
		ret_type: VarType,
		params: Vec<Value>,
	) -> Self {
		Self {
			cfg,
			name,
			ret_type,
			params,
		}
	}
}
