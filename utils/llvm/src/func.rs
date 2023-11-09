use crate::{label::Label, llvminstr::LlvmInstr, llvmvar::VarType, temp::Temp};

pub struct LlvmFunc {
	pub label: Label,
	pub ret_type: VarType,
	pub params: Vec<Temp>,
	pub body: Vec<Box<dyn LlvmInstr>>,
}
