use std::fmt::Display;
use utils::{InstrTrait, Label, UseTemp};

use crate::temp::Temp;

use super::{riscvop::*, value::*};

pub type RiscvInstr = Box<dyn RiscvInstrTrait>;

pub trait RiscvInstrTrait: Display + UseTemp<Temp> {}

impl UseTemp<Temp> for RiscvInstr {
	fn get_read(&self) -> Vec<Temp> {
		self.as_ref().get_read()
	}
	fn get_write(&self) -> Option<Temp> {
		self.as_ref().get_write()
	}
}

impl InstrTrait<Temp> for RiscvInstr {}

pub struct RTriInstr {
	pub op: RTriInstrOp,
	pub rd: RiscvTemp,
	pub rs1: RiscvTemp,
	pub rs2: RiscvTemp,
}

pub struct ITriInstr {
	pub op: ITriInstrOp,
	pub rd: RiscvTemp,
	pub rs1: RiscvTemp,
	pub rs2: RiscvImm,
}

pub struct IBinInstr {
	pub op: IBinInstrOp,
	pub rd: RiscvTemp,
	pub rs1: RiscvImm,
}

pub struct RBinInstr {
	pub op: RBinInstrOp,
	pub rd: RiscvTemp,
	pub rs1: RiscvTemp,
}

pub struct LabelInstr {
	pub label: Label,
}

pub struct BranInstr {
	pub op: BranInstrOp,
	pub rs1: RiscvTemp,
	pub rs2: RiscvTemp,
	pub to: RiscvImm,
}

pub struct NoArgInstr {
	pub op: NoArgInstrOp,
}
