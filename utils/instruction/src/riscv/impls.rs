#![allow(clippy::new_ret_no_self)]

use std::{collections::HashMap, fmt::Display};

use utils::Label;

use crate::temp::Temp;

use super::{
	reg::RiscvReg, riscvinstr::*, riscvop::*, utils::map_temp, value::*,
};

impl Display for RTriInstr {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "  {} {}, {}, {}", self.op, self.rd, self.rs1, self.rs2)
	}
}

impl RiscvInstrTrait for RTriInstr {
	fn map_temp(&mut self, map: &HashMap<Temp, RiscvReg>) {
		map_temp(&mut self.rd, map);
		map_temp(&mut self.rs1, map);
		map_temp(&mut self.rs2, map);
	}
	fn get_riscv_read(&self) -> Vec<RiscvTemp> {
		vec![self.rs1, self.rs2]
	}
	fn get_riscv_write(&self) -> Vec<RiscvTemp> {
		vec![self.rd]
	}
	fn is_move(&self) -> bool {
		self.op == Add && (self.rs1.is_zero() || self.rs2.is_zero())
	}
}

impl RTriInstr {
	pub fn new(
		op: RTriInstrOp,
		rd: RiscvTemp,
		rs1: RiscvTemp,
		rs2: RiscvTemp,
	) -> RiscvInstr {
		Box::new(Self {
			is_start: false,
			op,
			rs1,
			rs2,
			rd,
		})
	}
}

impl Display for ITriInstr {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "  {} {}, {}, {}", self.op, self.rd, self.rs1, self.rs2)
	}
}

impl RiscvInstrTrait for ITriInstr {
	fn map_temp(&mut self, map: &HashMap<Temp, RiscvReg>) {
		map_temp(&mut self.rd, map);
		map_temp(&mut self.rs1, map);
	}
	fn get_riscv_read(&self) -> Vec<RiscvTemp> {
		vec![self.rs1]
	}
	fn get_riscv_write(&self) -> Vec<RiscvTemp> {
		vec![self.rd]
	}
	fn is_move(&self) -> bool {
		self.op == Addi && !self.rs1.is_zero() && self.rs2.is_zero()
	}
}

impl ITriInstr {
	pub fn new(
		op: ITriInstrOp,
		rd: RiscvTemp,
		rs1: RiscvTemp,
		rs2: RiscvImm,
	) -> RiscvInstr {
		Box::new(Self {
			is_start: false,
			op,
			rs1,
			rs2,
			rd,
		})
	}
}

impl Display for IBinInstr {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "  {} {}, {}", self.op, self.rd, self.rs1)
	}
}

impl RiscvInstrTrait for IBinInstr {
	fn map_temp(&mut self, map: &HashMap<Temp, RiscvReg>) {
		map_temp(&mut self.rd, map);
	}
	fn get_riscv_write(&self) -> Vec<RiscvTemp> {
		match self.op {
			Li | Lui | LD | LW | LWU => vec![self.rd],
			SB | SH | SW | SD => vec![],
		}
	}
	fn get_riscv_read(&self) -> Vec<RiscvTemp> {
		match self.op {
			Li | Lui | LD | LW | LWU => vec![],
			SB | SH | SW | SD => vec![self.rd],
		}
	}
}

impl IBinInstr {
	pub fn new(op: IBinInstrOp, rd: RiscvTemp, rs1: RiscvImm) -> RiscvInstr {
		Box::new(Self {
			is_start: false,
			op,
			rs1,
			rd,
		})
	}
}

impl Display for LabelInstr {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{}:", self.label)
	}
}

impl RiscvInstrTrait for LabelInstr {}

impl LabelInstr {
	pub fn new(label: Label) -> RiscvInstr {
		Box::new(Self {
			is_start: false,
			label,
		})
	}
}

impl Display for RBinInstr {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "  {} {}, {}", self.op, self.rd, self.rs1)
	}
}

impl RiscvInstrTrait for RBinInstr {
	fn map_temp(&mut self, map: &HashMap<Temp, RiscvReg>) {
		map_temp(&mut self.rd, map);
		map_temp(&mut self.rs1, map);
	}
	fn get_riscv_read(&self) -> Vec<RiscvTemp> {
		vec![self.rs1]
	}
	fn get_riscv_write(&self) -> Vec<RiscvTemp> {
		vec![self.rd]
	}
}

impl RBinInstr {
	pub fn new(op: RBinInstrOp, rd: RiscvTemp, rs1: RiscvTemp) -> RiscvInstr {
		Box::new(Self {
			is_start: false,
			op,
			rs1,
			rd,
		})
	}
}

impl Display for BranInstr {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "  {} {}, {}, {}", self.op, self.rs1, self.rs2, self.to)
	}
}

impl RiscvInstrTrait for BranInstr {
	fn map_temp(&mut self, map: &HashMap<Temp, RiscvReg>) {
		map_temp(&mut self.rs1, map);
		map_temp(&mut self.rs2, map);
	}
	fn get_riscv_read(&self) -> Vec<RiscvTemp> {
		vec![self.rs1, self.rs2]
	}
}

impl BranInstr {
	pub fn new(
		op: BranInstrOp,
		rs1: RiscvTemp,
		rs2: RiscvTemp,
		to: RiscvImm,
	) -> RiscvInstr {
		Box::new(Self {
			is_start: false,
			op,
			rs1,
			rs2,
			to,
		})
	}
}

impl Display for NoArgInstr {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "  {}", self.op)
	}
}

impl RiscvInstrTrait for NoArgInstr {}

impl NoArgInstr {
	pub fn new(op: NoArgInstrOp) -> RiscvInstr {
		Box::new(Self {
			is_start: false,
			op,
		})
	}
}
