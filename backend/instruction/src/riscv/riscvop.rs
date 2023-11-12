use std::fmt::Display;

use sysyc_derive::Fuyuki;

use super::value::Value;

// trait

// type Reg = Box<dyn >
// type OffsetReg = (i32, )

impl Display for Value {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Imm(v) => write!(f, "{}", v),
			Self::Temp(v) => write!(f, "{}", v),
			Self::Reg(v) => write!(f, "{}", v),
		}
	}
}

#[derive(Fuyuki)]
pub enum TriInstrOp {
	Addi,
	Subi,
	Muli,
	Remi,
	Divi,
	Slti,
	Sltiu,
	Xori,
	Ori,
	Andi,
	Slli,
	Srli,
	Srai,

	Add,
	Sub,
	Mul,
	Rem,
	Div,
	Slt,
	Sltu,
	Xor,
	Or,
	And,
	Sll,
	Srl,
	Sra,

	#[style("fadd.s")]
	Fadd,
	#[style("fsub.s")]
	Fsub,
	#[style("fmul.s")]
	Fmul,
	#[style("fdiv.s")]
	Fdiv,
}
