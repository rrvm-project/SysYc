use crate::{label::Label, llvminstr::LlvmInstr, llvmvar::VarType};

pub struct LlvmFunc {
	pub label: Label,
	pub ret_type: VarType,
	pub body: Vec<Box<dyn LlvmInstr>>,
}
