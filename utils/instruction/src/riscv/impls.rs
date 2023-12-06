#![allow(clippy::new_ret_no_self)]

use std::{collections::HashMap, fmt::Display};

use utils::{Label, UseTemp};

use crate::temp::Temp;

use super::{
	reg::RiscvReg,
	riscvinstr::*,
	riscvop::*,
	utils::{map_temp, unwarp_temp, unwarp_temps},
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

impl RiscvInstrTrait for RTriInstr {
	fn map_temp(&mut self, map: &HashMap<Temp, RiscvReg>) {
		map_temp(&mut self.rd, map);
		map_temp(&mut self.rs1, map);
		map_temp(&mut self.rs2, map);
	}
}

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

impl RiscvInstrTrait for ITriInstr {
	fn map_temp(&mut self, map: &HashMap<Temp, RiscvReg>) {
		map_temp(&mut self.rd, map);
		map_temp(&mut self.rs1, map);
	}
}

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

impl RiscvInstrTrait for IBinInstr {
	fn map_temp(&mut self, map: &HashMap<Temp, RiscvReg>) {
		map_temp(&mut self.rd, map);
	}
}

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

impl RiscvInstrTrait for RBinInstr {
	fn map_temp(&mut self, map: &HashMap<Temp, RiscvReg>) {
		map_temp(&mut self.rd, map);
		map_temp(&mut self.rs1, map);
	}
}

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

impl RiscvInstrTrait for BranInstr {
	fn map_temp(&mut self, map: &HashMap<Temp, RiscvReg>) {
		map_temp(&mut self.rs1, map);
		map_temp(&mut self.rs2, map);
	}
}

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
