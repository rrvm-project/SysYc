use std::fmt::Display;
use utils::Label;

use super::{riscvop::*, value::*};

pub trait RiscvInstr: Display {}

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

pub struct ILoadInstr {
	pub op: BiLoadImmOp,
	pub rd: RiscvTemp,
	pub rs1: RiscvImm,
}

pub struct LabelInstr {
	pub label: Label,
}
