use crate::{label::Label, llvmop::*, llvmvar::VarType, temp::Temp};
use std::fmt::Display;

pub struct GlobalVar {}

pub trait LlvmInstr: Display {
	fn get_read(&self) -> Vec<Temp>;
	fn get_write(&self) -> Vec<Temp>;
	fn is_label(&self) -> bool;
	fn is_seq(&self) -> bool;
	fn type_valid(&self) -> bool;
}

pub struct ArithInstr {
	pub target: Temp,
	pub op: ArithOp,
	pub var_type: VarType,
	pub lhs: Value,
	pub rhs: Value,
}

pub struct LabelInstr {
	pub label: Label,
}

pub struct CompInstr {
	pub kind: CompKind,
	pub target: Temp,
	pub op: CompOp,
	pub var_type: VarType,
	pub lhs: Value,
	pub rhs: Value,
}

pub struct ConvertInstr {
	pub target: Temp,
	pub op: ConvertOp,
	pub var_type: VarType,
	pub lhs: Value,
	pub rhs: Value,
}
