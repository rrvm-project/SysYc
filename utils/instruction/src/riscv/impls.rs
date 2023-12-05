#![allow(clippy::new_ret_no_self)]

use std::fmt::Display;

use utils::{Label, UseTemp};

use crate::temp::Temp;

use super::{
	riscvinstr::*,
	riscvop::*,
	utils::{unwarp_temp, unwarp_temps},
	value::*,
};

impl Display for RTriInstr {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "    {} {}, {}, {}", self.op, self.rd, self.rs1, self.rs2)
	}
}

impl UseTemp<Temp> for RTriInstr {
	fn get_read(&self) -> Vec<Temp> {
		unwarp_temps(vec![&self.rs1, &self.rs2])
	}
	fn get_write(&self) -> Option<Temp> {
		unwarp_temp(&self.rd)
	}
}

impl RiscvInstrTrait for RTriInstr {}

impl RTriInstr {
	pub fn new(
		op: RTriInstrOp,
		rd: RiscvTemp,
		rs1: RiscvTemp,
		rs2: RiscvTemp,
	) -> RiscvInstr {
		Box::new(Self { op, rs1, rs2, rd })
	}
}

impl Display for ITriInstr {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "    {} {}, {}, {}", self.op, self.rd, self.rs1, self.rs2)
	}
}

impl UseTemp<Temp> for ITriInstr {
	fn get_read(&self) -> Vec<Temp> {
		unwarp_temps(vec![&self.rs1])
	}
	fn get_write(&self) -> Option<Temp> {
		unwarp_temp(&self.rd)
	}
}

impl RiscvInstrTrait for ITriInstr {}

impl ITriInstr {
	pub fn new(
		op: ITriInstrOp,
		rd: RiscvTemp,
		rs1: RiscvTemp,
		rs2: RiscvImm,
	) -> RiscvInstr {
		Box::new(Self { op, rs1, rs2, rd })
	}
}

impl Display for IBinInstr {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "    {} {}, {}", self.op, self.rd, self.rs1)
	}
}

impl UseTemp<Temp> for IBinInstr {
	fn get_write(&self) -> Option<Temp> {
		unwarp_temp(&self.rd)
	}
}

impl RiscvInstrTrait for IBinInstr {}

impl IBinInstr {
	pub fn new(op: IBinInstrOp, rd: RiscvTemp, rs1: RiscvImm) -> RiscvInstr {
		Box::new(Self { op, rs1, rd })
	}
}

impl Display for LabelInstr {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{}:", self.label)
	}
}

impl UseTemp<Temp> for LabelInstr {}

impl RiscvInstrTrait for LabelInstr {}

impl LabelInstr {
	pub fn new(label: Label) -> RiscvInstr {
		Box::new(Self { label })
	}
}

impl Display for RBinInstr {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "    {} {}, {}", self.op, self.rd, self.rs1)
	}
}

impl UseTemp<Temp> for RBinInstr {
	fn get_read(&self) -> Vec<Temp> {
		unwarp_temps(vec![&self.rs1])
	}
	fn get_write(&self) -> Option<Temp> {
		unwarp_temp(&self.rd)
	}
}

impl RiscvInstrTrait for RBinInstr {}

impl RBinInstr {
	pub fn new(op: RBinInstrOp, rd: RiscvTemp, rs1: RiscvTemp) -> RiscvInstr {
		Box::new(Self { op, rs1, rd })
	}
}

impl Display for BranInstr {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "    {} {}, {}, {}", self.op, self.rs1, self.rs2, self.to)
	}
}

impl UseTemp<Temp> for BranInstr {
	fn get_read(&self) -> Vec<Temp> {
		unwarp_temps(vec![&self.rs1, &self.rs2])
	}
}

impl RiscvInstrTrait for BranInstr {}

impl BranInstr {
	pub fn new(
		op: BranInstrOp,
		rs1: RiscvTemp,
		rs2: RiscvTemp,
		to: RiscvImm,
	) -> RiscvInstr {
		Box::new(Self { op, rs1, rs2, to })
	}
}

impl Display for NoArgInstr {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "    {}", self.op)
	}
}

impl UseTemp<Temp> for NoArgInstr {}

impl RiscvInstrTrait for NoArgInstr {}

impl NoArgInstr {
	pub fn new(op: NoArgInstrOp) -> RiscvInstr {
		Box::new(Self { op })
	}
}
