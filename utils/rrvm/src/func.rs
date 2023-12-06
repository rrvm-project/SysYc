use llvm::{Value, VarType};
use utils::{InstrTrait, TempTrait};

use crate::cfg::CFG;

pub struct RrvmFunc<T: InstrTrait<U>, U: TempTrait> {
	pub cfg: CFG<T, U>,
	pub name: String,
	pub ret_type: VarType,
	pub params: Vec<Value>,
}

impl<T: InstrTrait<U>, U: TempTrait> RrvmFunc<T, U> {
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
