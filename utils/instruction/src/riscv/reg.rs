use sysyc_derive::Fuyuki;
pub use RiscvReg::*;

pub const CALLER_SAVE: &[RiscvReg] = &[
	A0, A1, A2, A3, A4, A5, A6, A7, T0, T1, T2, T3, T4, T5, T6, RA,
];
pub const CALLEE_SAVE: &[RiscvReg] =
	&[FP, S1, S2, S3, S4, S5, S6, S7, S8, S9, S10, S11];
pub const ALLOCABLE_REGS: &[RiscvReg] = &[
	A0, A1, A2, A3, A4, A5, A6, A7, T0, T1, T2, T3, T4, T5, T6, S1, S2, S3, S4,
	S5, S6, S7, S8, S9, S10, S11,
];
pub const PARAMETER_REGS: &[RiscvReg] = &[A0, A1, A2, A3, A4, A5, A6, A7];

pub const ALLOACBLE_COUNT: usize = ALLOCABLE_REGS.len();

#[derive(Fuyuki, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RiscvReg {
	X0, // always zero
	RA, // return address
	SP, // stack pointer
	GP, // global pointer
	TP, // thread pointer
	T0,
	T1,
	T2,
	FP, // frame pointer
	S1,
	A0,
	A1,
	A2,
	A3,
	A4,
	A5,
	A6,
	A7,
	S2,
	S3,
	S4,
	S5,
	S6,
	S7,
	S8,
	S9,
	S10,
	S11,
	T3,
	T4,
	T5,
	T6,
}

impl RiscvReg {
	pub fn get_index(&self) -> Option<usize> {
		ALLOCABLE_REGS.iter().position(|&x| x == *self)
	}
}
