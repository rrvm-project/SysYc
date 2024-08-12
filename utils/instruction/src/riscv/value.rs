use std::fmt::Display;

use crate::temp::{Temp, VarType};

use super::{reg::RiscvReg, virt_mem::VirtAddr};
use utils::math::is_pow2;
pub use RiscvImm::*;
pub use RiscvTemp::*;

const RISCV_IMM_MAX: i32 = 2047;
const RISCV_IMM_MIN: i32 = -2048;

pub fn is_lower<T: std::cmp::Ord + From<i32>>(value: T) -> bool {
	let low = T::from(RISCV_IMM_MIN);
	let high = T::from(RISCV_IMM_MAX);
	value >= low && value <= high
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

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum RiscvImm {
	RiscvNumber(RiscvNumber),
	LongLong(i64),
	VirtMem(VirtAddr),
	Label(utils::Label),
	OffsetReg(RiscvNumber, RiscvTemp),
}
#[derive(Clone, Hash, PartialEq, Eq)]
pub enum RiscvNumber {
	Lo(utils::Label),
	Hi(utils::Label),
	Int(i32),
}
impl Display for RiscvNumber {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Lo(v) => write!(f, "%pcrel_lo({})", v),
			Self::Hi(v) => write!(f, "%pcrel_hi({})", v),
			Self::Int(v) => write!(f, "{}", v),
		}
	}
}
impl RiscvNumber {
	pub fn is_zero(&self) -> bool {
		matches!(self, RiscvNumber::Int(0))
	}
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
			Self::RiscvNumber(v) => write!(f, "{}", v),
			Self::Label(v) => write!(f, "{}", v),
			Self::LongLong(v) => write!(f, "{}", v),
			Self::VirtMem(v) => write!(f, "VirtMem[{}]", v.id),
			Self::OffsetReg(offset, base) => write!(f, "{}({})", offset, base),
		}
	}
}

impl From<i32> for RiscvImm {
	fn from(x: i32) -> Self {
		RiscvImm::RiscvNumber(RiscvNumber::Int(x))
	}
}

impl From<&i32> for RiscvImm {
	fn from(x: &i32) -> Self {
		RiscvImm::RiscvNumber(RiscvNumber::Int(*x))
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
		RiscvImm::OffsetReg(RiscvNumber::Int(x.0), x.1)
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
	pub fn get_type(&self) -> VarType {
		match self {
			VirtReg(v) => v.var_type,
			PhysReg(v) => v.get_type(),
		}
	}
}

impl RiscvImm {
	pub fn is_zero(&self) -> bool {
		matches!(self, RiscvImm::RiscvNumber(RiscvNumber::Int(0)))
	}
	pub fn to_virt_mem(&self) -> Option<VirtAddr> {
		if let RiscvImm::VirtMem(v) = self {
			Some(*v)
		} else {
			None
		}
	}
}
