use llvm::Value;
use sysyc_derive::Fuyuki;
pub use RiscvReg::*;

use crate::temp::VarType;

pub const CALLER_SAVE: &[RiscvReg] =
	&[A0, A1, A2, A3, A4, A5, A6, A7, T0, T1, T2, T3, T4, T5, T6];
pub const CALLEE_SAVE: &[RiscvReg] =
	&[FP, S1, S2, S3, S4, S5, S6, S7, S8, S9, S10, S11, RA];
pub const ALLOCABLE_REGS: &[RiscvReg] = &[
	A0, A1, A2, A3, A4, A5, A6, A7, T0, T1, T2, T3, T4, T5, T6, S1, S2, S3, S4,
	S5, S6, S7, S8, S9, S10, S11,
];

pub const FLOAT_CALLER_SAVE: &[RiscvReg] = &[
	Fa0, Fa1, Fa2, Fa3, Fa4, Fa5, Fa6, Fa7, Ft0, Ft1, Ft2, Ft3, Ft4, Ft5, Ft6,
	Ft7, Ft8, Ft9, Ft10, Ft11,
];
pub const FLOAT_CALLEE_SAVE: &[RiscvReg] =
	&[Fs0, Fs1, Fs2, Fs3, Fs4, Fs5, Fs6, Fs7, Fs8, Fs9, Fs10, Fs11];

const PARAMETER_REGS: &[RiscvReg] = &[A0, A1, A2, A3, A4, A5, A6, A7];
const FLOAT_PARAMETER_REGS: &[RiscvReg] =
	&[Fa0, Fa1, Fa2, Fa3, Fa4, Fa5, Fa6, Fa7];

pub fn alloc_params_register(
	params: Vec<Value>,
) -> (Vec<(Value, RiscvReg)>, Vec<Value>) {
	let mut regs = Vec::new();
	let mut stack = Vec::new();
	let mut int_cnt = 0;
	let mut float_cnt = 0;

	for param in params {
		match param.get_type() {
			llvm::VarType::I32 | llvm::VarType::I32Ptr | llvm::VarType::F32Ptr => {
				if int_cnt < PARAMETER_REGS.len() {
					regs.push((param, PARAMETER_REGS[int_cnt]));
					int_cnt += 1;
				} else {
					stack.push(param);
				}
			}
			llvm::VarType::F32 => {
				if float_cnt < FLOAT_PARAMETER_REGS.len() {
					regs.push((param, FLOAT_PARAMETER_REGS[float_cnt]));
					float_cnt += 1;
				} else {
					stack.push(param);
				}
			}
			_ => unreachable!(),
		}
	}
	(regs, stack)
}

#[derive(Fuyuki, Clone, Copy, PartialEq, Eq, Hash, Debug)]
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
	// floating point registers
	Ft0,
	Ft1,
	Ft2,
	Ft3,
	Ft4,
	Ft5,
	Ft6,
	Ft7,
	Fs0,
	Fs1,
	Fa0,
	Fa1,
	Fa2,
	Fa3,
	Fa4,
	Fa5,
	Fa6,
	Fa7,
	Fs2,
	Fs3,
	Fs4,
	Fs5,
	Fs6,
	Fs7,
	Fs8,
	Fs9,
	Fs10,
	Fs11,
	Ft8,
	Ft9,
	Ft10,
	Ft11,
}

impl RiscvReg {
	pub fn get_type(&self) -> VarType {
		match self {
			X0 | RA | SP | GP | TP | T0 | T1 | T2 | FP | S1 | A0 | A1 | A2 | A3
			| A4 | A5 | A6 | A7 | S2 | S3 | S4 | S5 | S6 | S7 | S8 | S9 | S10
			| S11 | T3 | T4 | T5 | T6 => VarType::Int,
			Ft0 | Ft1 | Ft2 | Ft3 | Ft4 | Ft5 | Ft6 | Ft7 | Fs0 | Fs1 | Fa0 | Fa1
			| Fa2 | Fa3 | Fa4 | Fa5 | Fa6 | Fa7 | Fs2 | Fs3 | Fs4 | Fs5 | Fs6
			| Fs7 | Fs8 | Fs9 | Fs10 | Fs11 | Ft8 | Ft9 | Ft10 | Ft11 => VarType::Float,
		}
	}
}
