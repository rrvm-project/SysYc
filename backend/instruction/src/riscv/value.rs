use std::fmt::Display;

use crate::temp::Temp;

use super::reg::RiscvReg;
pub use RiscvImm::*;
pub use RiscvTemp::*;

const RISCV_IMM_MAX: i32 = 2047;
const RISCV_IMM_MIN: i32 = -2048;

pub fn is_lower(x: i32) -> bool {
	RISCV_IMM_MIN < x && x < RISCV_IMM_MAX
}

#[derive(Clone, Copy)]
pub enum RiscvTemp {
	VirtReg(Temp),
	PhysReg(RiscvReg),
}

impl From<Temp> for RiscvTemp {
	fn from(x: Temp) -> Self {
		RiscvTemp::VirtReg(x)
	}
}

impl From<RiscvReg> for RiscvTemp {
	fn from(x: RiscvReg) -> Self {
		RiscvTemp::PhysReg(x)
	}
}

pub enum RiscvImm {
	Int(i32),
	Label(utils::Label),
	OffsetReg(i32, RiscvTemp),
}

impl Display for RiscvTemp {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::VirtReg(v) => write!(f, "{}", v),
			Self::PhysReg(v) => write!(f, "{}", v),
		}
	}
}

impl Display for RiscvImm {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Int(v) => write!(f, "{}", v),
			Self::Label(v) => write!(f, "{}", v),
			Self::OffsetReg(offset, base) => write!(f, "{}({})", offset, base),
		}
	}
}

impl From<i32> for RiscvImm {
	fn from(x: i32) -> Self {
		RiscvImm::Int(x)
	}
}

impl From<utils::Label> for RiscvImm {
	fn from(x: utils::Label) -> Self {
		RiscvImm::Label(x)
	}
}

impl From<(i32, RiscvTemp)> for RiscvImm {
	fn from(x: (i32, RiscvTemp)) -> Self {
		RiscvImm::OffsetReg(x.0, x.1)
	}
}
