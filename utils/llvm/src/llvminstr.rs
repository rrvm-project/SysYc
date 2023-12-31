use utils::{InstrTrait, Label, UseTemp};

use crate::{llvmop::*, LlvmInstrVariant, LlvmTemp, VarType};
use std::{collections::HashMap, fmt::Display};

pub type LlvmInstr = Box<dyn LlvmInstrTrait>;

pub trait CloneLlvmInstr {
	fn clone_box(&self) -> LlvmInstr;
}

impl<T> CloneLlvmInstr for T
where
	T: 'static + LlvmInstrTrait + Clone,
{
	fn clone_box(&self) -> LlvmInstr {
		Box::new(self.clone())
	}
}

impl Clone for LlvmInstr {
	fn clone(&self) -> Self {
		self.clone_box()
	}
}

pub trait LlvmInstrTrait: Display + CloneLlvmInstr + UseTemp<LlvmTemp> {
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
	fn get_label(&self) -> Label {
		unreachable!()
	}
	fn is_load(&self) -> bool {
		false
	}
	fn is_store(&self) -> bool {
		false
	}
	fn is_call(&self) -> bool {
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
	fn get_alloc(&self) -> Option<(LlvmTemp, Value)> {
		None
	}
	fn replace_read(&mut self, _old: Temp, _new: Value) {}
	fn map_temp(&mut self, _map: &HashMap<Temp, Value>) {}
	fn set_target(&mut self, _target: LlvmTemp) {}
	fn map_label(&mut self, _map: &HashMap<Label, Label>) {
		unreachable!()
	}
	fn replaceable(&self, _map: &HashMap<Temp, Value>) -> bool {
		false
	}
	fn is_candidate_operator(&self) -> Option<ArithOp> {
		None
	}
	fn get_lhs_and_rhs(&self) -> Option<(Value, Value)> {
		None
	}
}

impl UseTemp<LlvmTemp> for LlvmInstr {
	fn get_read(&self) -> Vec<LlvmTemp> {
		self.as_ref().get_read()
	}
	fn get_write(&self) -> Option<LlvmTemp> {
		self.as_ref().get_write()
	}
}

impl InstrTrait<LlvmTemp> for LlvmInstr {
	fn is_call(&self) -> bool {
		self.as_ref().is_call()
	}
}

#[derive(Clone)]
pub struct ArithInstr {
	pub target: LlvmTemp,
	pub op: ArithOp,
	pub var_type: VarType,
	pub lhs: Value,
	pub rhs: Value,
}
#[derive(Clone)]
pub struct CompInstr {
	pub kind: CompKind,
	pub target: LlvmTemp,
	pub op: CompOp,
	pub var_type: VarType,
	pub lhs: Value,
	pub rhs: Value,
}

#[derive(Clone)]
pub struct ConvertInstr {
	pub target: LlvmTemp,
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
	pub target: LlvmTemp,
	pub var_type: VarType,
	pub source: Vec<(Value, Label)>,
}

#[derive(Clone)]
pub struct RetInstr {
	pub value: Option<Value>,
}

#[derive(Clone)]
pub struct AllocInstr {
	pub target: LlvmTemp,
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
	pub target: LlvmTemp,
	pub var_type: VarType,
	pub addr: Value,
}

#[derive(Clone)]
pub struct GEPInstr {
	pub target: LlvmTemp,
	pub var_type: VarType,
	pub addr: Value,
	pub offset: Value,
}

#[derive(Clone)]
pub struct CallInstr {
	pub target: LlvmTemp,
	pub var_type: VarType,
	pub func: Label,
	pub params: Vec<(VarType, Value)>,
}
