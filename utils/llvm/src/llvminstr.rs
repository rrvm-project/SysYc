use utils::{InstrTrait, Label, UseTemp};

use crate::{
	llvminstrattr::{LlvmAttr, LlvmAttrs},
	llvmop::*,
	llvmvar::VarType,
	LlvmInstrVariant, Temp,
};
use std::{collections::HashMap, fmt::Display};

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

pub trait LlvmInstrTrait:
	Display + CloneLlvmInstr + UseTemp<Temp> + LlvmAttrs
{
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
	fn has_sideeffect(&self) -> bool {
		false
	}
	fn is_ret(&self) -> bool {
		false
	}
	fn is_jump_cond(&self) -> bool {
		false
	}
	fn is_direct_jump(&self) -> bool {
		false
	}
	fn is_call(&self) -> bool {
		false
	}
	fn get_alloc(&self) -> Option<(Temp, Value)> {
		None
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
	pub _attrs: HashMap<String, LlvmAttr>,
}
#[derive(Clone)]
pub struct CompInstr {
	pub kind: CompKind,
	pub target: Temp,
	pub op: CompOp,
	pub var_type: VarType,
	pub lhs: Value,
	pub rhs: Value,
	pub _attrs: HashMap<String, LlvmAttr>,
}

#[derive(Clone)]
pub struct ConvertInstr {
	pub target: Temp,
	pub op: ConvertOp,
	pub from_type: VarType,
	pub lhs: Value,
	pub to_type: VarType,
	pub _attrs: HashMap<String, LlvmAttr>,
}

#[derive(Clone)]
pub struct JumpInstr {
	pub target: Label,
	pub _attrs: HashMap<String, LlvmAttr>,
}

#[derive(Clone)]
pub struct JumpCondInstr {
	pub var_type: VarType,
	pub cond: Value,
	pub target_true: Label,
	pub target_false: Label,
	pub _attrs: HashMap<String, LlvmAttr>,
}

#[derive(Clone)]
pub struct PhiInstr {
	pub target: Temp,
	pub var_type: VarType,
	pub source: Vec<(Value, Label)>,
	pub _attrs: HashMap<String, LlvmAttr>,
}

#[derive(Clone)]
pub struct RetInstr {
	pub value: Option<Value>,
	pub _attrs: HashMap<String, LlvmAttr>,
}

#[derive(Clone)]
pub struct AllocInstr {
	pub target: Temp,
	pub var_type: VarType,
	pub length: Value,
	pub _attrs: HashMap<String, LlvmAttr>,
}

#[derive(Clone)]
pub struct StoreInstr {
	pub value: Value,
	pub addr: Value,
	pub _attrs: HashMap<String, LlvmAttr>,
}

#[derive(Clone)]
pub struct LoadInstr {
	pub target: Temp,
	pub var_type: VarType,
	pub addr: Value,
	pub _attrs: HashMap<String, LlvmAttr>,
}

#[derive(Clone)]
pub struct GEPInstr {
	pub target: Temp,
	pub var_type: VarType,
	pub addr: Value,
	pub offset: Value,
	pub _attrs: HashMap<String, LlvmAttr>,
}

#[derive(Clone)]
pub struct CallInstr {
	pub target: Temp,
	pub var_type: VarType,
	pub func: Label,
	pub params: Vec<(VarType, Value)>,
	pub _attrs: HashMap<String, LlvmAttr>,
}
