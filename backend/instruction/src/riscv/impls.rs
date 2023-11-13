#![allow(clippy::new_ret_no_self)]

use std::fmt::Display;

use super::{riscvinstr::*, riscvop::*, value::*};

impl Display for RTriInstr {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "    {} {}, {}, {}", self.op, self.rd, self.rs1, self.rs2)
	}
}

impl RiscvInstr for RTriInstr {}

impl RTriInstr {
	pub fn new(
		op: RTriInstrOp,
		rd: RiscvTemp,
		rs1: RiscvTemp,
		rs2: RiscvTemp,
	) -> Box<dyn RiscvInstr> {
		Box::new(Self { op, rs1, rs2, rd })
	}
}

impl Display for ITriInstr {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "    {} {}, {}, {}", self.op, self.rd, self.rs1, self.rs2)
	}
}

impl RiscvInstr for ITriInstr {}

impl ITriInstr {
	pub fn new(
		op: ITriInstrOp,
		rd: RiscvTemp,
		rs1: RiscvTemp,
		rs2: RiscvImm,
	) -> Box<dyn RiscvInstr> {
		Box::new(Self { op, rs1, rs2, rd })
	}
}

impl Display for ILoadInstr {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "    {} {}, {}", self.op, self.rd, self.rs1)
	}
}

impl RiscvInstr for ILoadInstr {}

impl ILoadInstr {
	pub fn new(
		op: BiLoadImmOp,
		rd: RiscvTemp,
		rs1: RiscvImm,
	) -> Box<dyn RiscvInstr> {
		Box::new(Self { op, rs1, rd })
	}
}
