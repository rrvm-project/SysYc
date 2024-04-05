use std::fmt::Display;

use crate::temp::Temp;

use super::{reg::RiscvReg, virt_mem::VirtAddr};
use utils::math::is_pow2;
pub use RiscvImm::*;
pub use RiscvTemp::*;

const RISCV_IMM_MAX: i32 = 2047;
const RISCV_IMM_MIN: i32 = -2048;

pub fn is_lower(x: i32) -> bool {
	RISCV_IMM_MIN < x && x < RISCV_IMM_MAX
}

pub fn can_optimized_mul(x: i32) -> bool {
	let x = x.abs() >> x.abs().trailing_zeros();
	is_pow2(x) || is_pow2(x - 1) || is_pow2(x + 1)
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
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

#[derive(Clone)]
pub enum RiscvImm {
	Int(i32),
	LongLong(i64),
	VirtMem(VirtAddr),
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
			Self::LongLong(v) => write!(f, "{}", v),
			Self::VirtMem(v) => write!(f, "VirtMem[{}]", v.id),
			Self::OffsetReg(offset, base) => write!(f, "{}({})", offset, base),
		}
	}
}

impl From<i32> for RiscvImm {
	fn from(x: i32) -> Self {
		RiscvImm::Int(x)
	}
}

impl From<i64> for RiscvImm {
	fn from(x: i64) -> Self {
		RiscvImm::LongLong(x)
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

impl From<VirtAddr> for RiscvImm {
	fn from(value: VirtAddr) -> Self {
		RiscvImm::VirtMem(value)
	}
}

impl RiscvTemp {
	pub fn is_zero(&self) -> bool {
		matches!(self, PhysReg(RiscvReg::X0))
	}
	pub fn is_virtual(&self) -> bool {
		matches!(self, VirtReg(_))
	}
	pub fn get_phys(&self) -> Option<RiscvReg> {
		match self {
			VirtReg(_) => None,
			PhysReg(v) => Some(*v),
		}
	}
}

impl RiscvImm {
	pub fn is_zero(&self) -> bool {
		matches!(self, Int(0))
	}
	pub fn to_virt_mem(&self) -> Option<VirtAddr> {
		if let RiscvImm::VirtMem(v) = self {
			Some(*v)
		} else {
			None
		}
	}
}
