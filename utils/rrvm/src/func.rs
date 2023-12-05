use std::fmt::Display;

use llvm::{Value, VarType};
use utils::UseTemp;

use crate::cfg::CFG;

pub struct RrvmFunc<T: Display + UseTemp<U>, U: Display> {
	pub cfg: CFG<T, U>,
	pub name: String,
	pub ret_type: VarType,
	pub params: Vec<Value>,
}

impl<T: Display + UseTemp<U>, U: Display> RrvmFunc<T, U> {
	pub fn new(
		cfg: CFG<T, U>,
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
