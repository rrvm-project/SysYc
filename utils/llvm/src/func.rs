use utils::Label;

use crate::{llvminstr::LlvmInstr, llvmvar::VarType, temp::Temp};
use std::fmt::Display;

pub struct LlvmFunc {
	pub label: Label,
	pub ret_type: VarType,
	pub params: Vec<Temp>,
	pub body: Vec<Box<dyn LlvmInstr>>,
}

impl Display for LlvmFunc {
	fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		todo!()
	}
}
