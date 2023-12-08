use std::{collections::HashMap, fmt::Display};
use sysyc_derive::{has_riscv_attrs, UseTemp};
use utils::{mapper::LabelMapper, InstrTrait, Label, UseTemp};

use crate::temp::Temp;

use super::{reg::RiscvReg, riscvop::*, value::*};

pub type RiscvInstr = Box<dyn RiscvInstrTrait>;

pub trait RiscvAttr {
	fn is_start(&self) -> bool;
	fn set_start(&mut self, val: bool) -> bool;
}

pub trait RiscvInstrTrait: Display + UseTemp<Temp> + RiscvAttr {
	fn map_temp(&mut self, _map: &HashMap<Temp, RiscvReg>) {}
	fn get_riscv_read(&self) -> Vec<RiscvTemp> {
		Vec::new()
	}
	fn get_riscv_write(&self) -> Vec<RiscvTemp> {
		Vec::new()
	}
	fn is_move(&self) -> bool {
		false
	}
	fn get_label(&self) -> Label {
		unreachable!()
	}
	fn map_label(&mut self, _map: &mut LabelMapper) {}
}

impl UseTemp<Temp> for RiscvInstr {
	fn get_read(&self) -> Vec<Temp> {
		self.as_ref().get_read()
	}
	fn get_write(&self) -> Option<Temp> {
		self.as_ref().get_write()
	}
}

impl InstrTrait<Temp> for RiscvInstr {}

#[has_riscv_attrs]
#[derive(UseTemp)]
pub struct RTriInstr {
	pub op: RTriInstrOp,
	pub rd: RiscvTemp,
	pub rs1: RiscvTemp,
	pub rs2: RiscvTemp,
}

#[has_riscv_attrs]
#[derive(UseTemp)]
pub struct ITriInstr {
	pub op: ITriInstrOp,
	pub rd: RiscvTemp,
	pub rs1: RiscvTemp,
	pub rs2: RiscvImm,
}

#[has_riscv_attrs]
#[derive(UseTemp)]
pub struct IBinInstr {
	pub op: IBinInstrOp,
	pub rd: RiscvTemp,
	pub rs1: RiscvImm,
}

#[has_riscv_attrs]
#[derive(UseTemp)]
pub struct RBinInstr {
	pub op: RBinInstrOp,
	pub rd: RiscvTemp,
	pub rs1: RiscvTemp,
}

#[has_riscv_attrs]
#[derive(UseTemp)]
pub struct LabelInstr {
	pub label: Label,
}

#[has_riscv_attrs]
#[derive(UseTemp)]
pub struct BranInstr {
	pub op: BranInstrOp,
	pub rs1: RiscvTemp,
	pub rs2: RiscvTemp,
	pub to: RiscvImm,
}

#[has_riscv_attrs]
#[derive(UseTemp)]
pub struct NoArgInstr {
	pub op: NoArgInstrOp,
}
