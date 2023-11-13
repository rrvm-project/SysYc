use sysyc_derive::Fuyuki;

/// op rd, rs1, imm
#[derive(Fuyuki)]
pub enum ITriInstrOp {
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
}

/// op rd, rs1, rs2
#[derive(Fuyuki)]
pub enum RTriInstrOp {
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

/// op rd, imm
#[derive(Fuyuki)]
pub enum BiLoadImmOp {
	Li,
	Lui,
}

#[derive(Fuyuki)]
pub enum UnInstrOp {
	Li,
	Lb,
	Lh,
}
