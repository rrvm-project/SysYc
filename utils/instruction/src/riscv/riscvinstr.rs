use std::{collections::HashMap, fmt::Display};
use sysyc_derive::UseTemp;
use utils::{mapper::LabelMapper, InstrTrait, Label, UseTemp};

use crate::temp::Temp;

use super::{riscvop::*, value::*};

pub type RiscvInstr = Box<dyn RiscvInstrTrait>;

pub trait CloneRiscvInstr {
	fn clone_box(&self) -> Box<dyn RiscvInstrTrait>;
}

impl<T> CloneRiscvInstr for T
where
	T: 'static + RiscvInstrTrait + Clone,
{
	fn clone_box(&self) -> Box<dyn RiscvInstrTrait> {
		Box::new(self.clone())
	}
}

pub trait RiscvInstrTrait: Display + UseTemp<Temp> + CloneRiscvInstr {
	fn map_temp(&mut self, _map: &HashMap<Temp, RiscvTemp>) {}
	fn get_riscv_read(&self) -> Vec<RiscvTemp> {
		Vec::new()
	}
	fn get_riscv_write(&self) -> Vec<RiscvTemp> {
		Vec::new()
	}
	fn get_read_label(&self) -> Option<Label> {
		None
	}
	fn get_write_label(&self) -> Option<Label> {
		None
	}
	fn is_move(&self) -> bool {
		false
	}
	fn is_ret(&self) -> bool {
		false
	}
	fn is_call(&self) -> bool {
		false
	}
	fn map_label(&mut self, _map: &mut LabelMapper) {}
	fn useless(&self) -> bool {
		false
	}
	fn get_temp_op(&self) -> Option<TemporayInstrOp> {
		None
	}
}

impl UseTemp<Temp> for RiscvInstr {
	fn get_read(&self) -> Vec<Temp> {
		self.as_ref().get_read()
	}
	fn get_write(&self) -> Option<Temp> {
		self.as_ref().get_write()
	}
}

impl InstrTrait<Temp> for RiscvInstr {
	fn is_call(&self) -> bool {
		self.as_ref().is_call()
	}
}

#[derive(UseTemp, Clone)]
pub struct RTriInstr {
	pub op: RTriInstrOp,
	pub rd: RiscvTemp,
	pub rs1: RiscvTemp,
	pub rs2: RiscvTemp,
}

#[derive(UseTemp, Clone)]
pub struct ITriInstr {
	pub op: ITriInstrOp,
	pub rd: RiscvTemp,
	pub rs1: RiscvTemp,
	pub rs2: RiscvImm,
}

#[derive(UseTemp, Clone)]
pub struct IBinInstr {
	pub op: IBinInstrOp,
	pub rd: RiscvTemp,
	pub rs1: RiscvImm,
}

#[derive(UseTemp, Clone)]
pub struct RBinInstr {
	pub op: RBinInstrOp,
	pub rd: RiscvTemp,
	pub rs1: RiscvTemp,
}

#[derive(UseTemp, Clone)]
pub struct LabelInstr {
	pub label: Label,
}

#[derive(UseTemp, Clone)]
pub struct BranInstr {
	pub op: BranInstrOp,
	pub rs1: RiscvTemp,
	pub rs2: RiscvTemp,
	pub to: RiscvImm,
}

#[derive(UseTemp, Clone)]
pub struct NoArgInstr {
	pub op: NoArgInstrOp,
}

#[derive(UseTemp, Clone)]
pub struct CallInstr {
	pub func_label: Label,
	pub params: Vec<RiscvTemp>,
}

#[derive(UseTemp, Clone)]
pub struct TemporayInstr {
	pub op: TemporayInstrOp,
}
