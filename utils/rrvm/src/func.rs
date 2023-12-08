use llvm::{Value, VarType};
use utils::{InstrTrait, TempTrait};

use crate::cfg::CFG;

pub struct RrvmFunc<T: InstrTrait<U>, U: TempTrait> {
	pub total: u32,
	pub spill_size: i32,
	pub cfg: CFG<T, U>,
	pub name: String,
	pub ret_type: VarType,
	pub params: Vec<Value>,
}

impl<T: InstrTrait<U>, U: TempTrait> RrvmFunc<T, U> {}
