use utils::{InstrTrait, Label, UseTemp};

use crate::{llvmop::*, llvmvar::VarType, LlvmInstrVariant, Temp};
use std::fmt::Display;

pub type LlvmInstr = Box<dyn LlvmInstrTrait>;

pub trait CloneLlvmInstr {
	fn clone_box(&self) -> Box<dyn LlvmInstrTrait>;
}

impl<T> CloneLlvmInstr for T
where
	T: 'static + LlvmInstrTrait + Clone,
{
	fn clone_box(&self) -> Box<dyn LlvmInstrTrait> {
		Box::new(self.clone())
	}
}

pub trait LlvmInstrTrait: Display + CloneLlvmInstr + UseTemp<Temp> {
	fn type_valid(&self) -> bool {
		true
	}
	fn is_phi(&self) -> bool {
		false
	}
	fn get_succ(&self) -> Vec<Label> {
		Vec::new()
	}
	fn get_variant(&self) -> LlvmInstrVariant;
	fn new_jump(&self) -> Option<JumpInstr> {
		None
	}
	fn is_load(&self) -> bool {
		false
	}
	fn is_store(&self) -> bool {
		false
	}
}

impl UseTemp<Temp> for LlvmInstr {
	fn get_read(&self) -> Vec<Temp> {
		self.as_ref().get_read()
	}
	fn get_write(&self) -> Option<Temp> {
		self.as_ref().get_write()
	}
}

impl InstrTrait<Temp> for LlvmInstr {}

#[derive(Clone)]
pub struct ArithInstr {
	pub target: Temp,
	pub op: ArithOp,
	pub var_type: VarType,
	pub lhs: Value,
	pub rhs: Value,
}

#[derive(Clone)]
pub struct CompInstr {
	pub kind: CompKind,
	pub target: Temp,
	pub op: CompOp,
	pub var_type: VarType,
	pub lhs: Value,
	pub rhs: Value,
}

#[derive(Clone)]
pub struct ConvertInstr {
	pub target: Temp,
	pub op: ConvertOp,
	pub from_type: VarType,
	pub lhs: Value,
	pub to_type: VarType,
}

#[derive(Clone)]
pub struct JumpInstr {
	pub target: Label,
}

#[derive(Clone)]
pub struct JumpCondInstr {
	pub var_type: VarType,
	pub cond: Value,
	pub target_true: Label,
	pub target_false: Label,
}

#[derive(Clone)]
pub struct PhiInstr {
	pub target: Temp,
	pub var_type: VarType,
	pub source: Vec<(Value, Label)>,
}

#[derive(Clone)]
pub struct RetInstr {
	pub value: Option<Value>,
}

#[derive(Clone)]
pub struct AllocInstr {
	pub target: Temp,
	pub var_type: VarType,
	pub length: Value,
}

#[derive(Clone)]
pub struct StoreInstr {
	pub value: Value,
	pub addr: Value,
}

#[derive(Clone)]
pub struct LoadInstr {
	pub target: Temp,
	pub var_type: VarType,
	pub addr: Value,
}

#[derive(Clone)]
pub struct GEPInstr {
	pub target: Temp,
	pub var_type: VarType,
	pub addr: Value,
	pub offset: Value,
}

#[derive(Clone)]
pub struct CallInstr {
	pub target: Temp,
	pub var_type: VarType,
	pub func: Label,
	pub params: Vec<(VarType, Value)>,
}
