use llvm::{Value, VarType};
use utils::{
	InstrTrait, TempTrait, INLINE_PARAMS_THRESHOLD, MAX_INLINE_LENGTH,
};

use crate::{
	cfg::{BasicBlock, CFG},
	prelude::LlvmFunc,
};

pub struct RrvmFunc<T: InstrTrait<U>, U: TempTrait> {
	pub total: i32, // 用于创建新基本块，记录了已经创建过的基本块数量，total+1为下一个基本块的编号。注意：不等于cfg.blocks.len(),因为编号中间可能有基本块被删除了
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
	pub fn new_basicblock(&mut self, weight: f64) -> BasicBlock<T, U> {
		self.total += 1;
		BasicBlock::new(self.total, weight)
	}
}

impl LlvmFunc {
	pub fn is_recursive(&self) -> bool {
		self.cfg.blocks.iter().any(|v| {
			v.borrow()
				.instrs
				.iter()
				.any(|v| v.is_call() && v.get_label().name == self.name)
		})
	}
	pub fn can_inline(&self) -> bool {
		!self.is_recursive()
			&& (self.len() < MAX_INLINE_LENGTH
				|| self.params.len() > INLINE_PARAMS_THRESHOLD)
	}
}
