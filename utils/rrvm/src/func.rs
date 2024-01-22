use llvm::{Value, VarType};
use utils::{
	InstrTrait, TempTrait, INLINE_PARAMS_THRESHOLD, MAX_INLINE_LENGTH,
};

use crate::cfg::CFG;

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
	pub fn len(&self) -> usize {
		self.cfg.blocks.iter().map(|v| v.borrow().instrs.len()).sum()
	}
	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}
	pub fn can_inline(&self) -> bool {
		self.is_leaf()
			&& (self.len() < MAX_INLINE_LENGTH
				|| self.params.len() > INLINE_PARAMS_THRESHOLD)
	}
}
