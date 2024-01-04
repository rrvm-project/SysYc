use llvm::{Value, VarType};
use utils::{InstrTrait, TempTrait};

use crate::{cfg::CFG, program::RiscvFunc};

pub struct RrvmFunc<T: InstrTrait<U>, U: TempTrait> {
	pub total: i32,
	pub spills: i32,
	pub cfg: CFG<T, U>,
	pub name: String,
	pub ret_type: VarType,
	pub params: Vec<Value>,
}

impl<T: InstrTrait<U>, U: TempTrait> RrvmFunc<T, U> {
	pub fn is_leaf(&self) -> bool {
		!self
			.cfg
			.blocks
			.iter()
			.any(|v| v.borrow().instrs.iter().any(|v| v.is_call()))
	}
}

impl RiscvFunc {
	pub fn max_temp(&self) -> i32 {
		self
			.cfg
			.blocks
			.iter()
			.filter_map(|v| {
				v.borrow()
					.instrs
					.iter()
					.map(|v| v.get_write().map_or(0, |v| v.id))
					.max()
			})
			.max()
			.unwrap_or_default()
	}
}
