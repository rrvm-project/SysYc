use llvm::temp::Temp;
use sysyc_derive::FuyukiDisplay;

#[derive(FuyukiDisplay)]
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

pub enum Value {
	Imm(i32),
	Temp(Temp),
	Register(RiscvReg),
}
