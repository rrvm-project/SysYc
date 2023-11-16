#![allow(clippy::new_ret_no_self)]

use std::fmt::Display;

use utils::Label;

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

impl Display for IBinInstr {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "    {} {}, {}", self.op, self.rd, self.rs1)
	}
}

impl RiscvInstr for IBinInstr {}

impl IBinInstr {
	pub fn new(
		op: IBinInstrOp,
		rd: RiscvTemp,
		rs1: RiscvImm,
	) -> Box<dyn RiscvInstr> {
		Box::new(Self { op, rs1, rd })
	}
}

impl Display for LabelInstr {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{}:", self.label)
	}
}

impl RiscvInstr for LabelInstr {}

impl LabelInstr {
	pub fn new(label: Label) -> Box<dyn RiscvInstr> {
		Box::new(Self { label })
	}
}

impl Display for RBinInstr {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "    {} {}, {}", self.op, self.rd, self.rs1)
	}
}

impl RiscvInstr for RBinInstr {}

impl RBinInstr {
	pub fn new(
		op: RBinInstrOp,
		rd: RiscvTemp,
		rs1: RiscvTemp,
	) -> Box<dyn RiscvInstr> {
		Box::new(Self { op, rs1, rd })
	}
}

impl Display for BranInstr {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "    {} {}, {}, {}", self.op, self.rs1, self.rs2, self.to)
	}
}

impl RiscvInstr for BranInstr {}

impl BranInstr {
	pub fn new(
		op: BranInstrOp,
		rs1: RiscvTemp,
		rs2: RiscvTemp,
		to: RiscvImm,
	) -> Box<dyn RiscvInstr> {
		Box::new(Self { op, rs1, rs2, to })
	}
}

impl Display for NoArgInstr {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "    {}", self.op)
	}
}

impl RiscvInstr for NoArgInstr {}

impl NoArgInstr {
	pub fn new(op: NoArgInstrOp) -> Box<dyn RiscvInstr> {
		Box::new(Self { op })
	}
}