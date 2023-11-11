use llvm::temp::Temp;
use sysyc_derive::Fuyuki;

use crate::reg::RiscvReg;

pub enum Value {
	Imm(i32),
	Temp(Temp),
	Register(RiscvReg),
}

#[derive(Fuyuki)]
pub enum ITypeOp {
	Addi,
	Alti,
	Sltiu,
	Xori,
	Ori,

	Andi,
	Slli,
	Srli,
	Srai,

	Lb,
	Lh,
	Lw,
	Lbu,
	Lhu,
}
