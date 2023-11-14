use llvm::llvmop::ArithOp;
use sysyc_derive::Fuyuki;

pub use BiLoadImmOp::*;
pub use ITriInstrOp::*;
pub use RTriInstrOp::*;

/// op rd, rs1, imm
#[derive(Fuyuki)]
pub enum ITriInstrOp {
	Addi,
	Subi,
	Muli,
	Remi,
	Divi,
	Xori,
	Ori,
	Andi,
	Slli,
	Srli,
	Srai,
	Slti,
	Sltiu,
}

/// op rd, rs1, rs2
#[derive(Fuyuki)]
pub enum RTriInstrOp {
	Add,
	Sub,
	Mul,
	Rem,
	Div,
	Xor,
	Or,
	And,
	Sll,
	Srl,
	Sra,
	Slt,
	Sltu,

	#[style("fadd.s")]
	Fadd,
	#[style("fsub.s")]
	Fsub,
	#[style("fmul.s")]
	Fmul,
	#[style("fdiv.s")]
	Fdiv,

	#[style("feq.s")]
	Feq,
	#[style("flt.s")]
	Flt,
	#[style("fle.s")]
	Fle,
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

pub fn to_iop(op: &ArithOp) -> ITriInstrOp {
	match op {
		ArithOp::Add => Addi,
		ArithOp::Sub => Subi,
		ArithOp::Mul => Muli,
		ArithOp::Div => Divi,
		ArithOp::Rem => Remi,
		ArithOp::Shl => Slli,
		ArithOp::Lshr => Srli,
		ArithOp::Ashr => Srai,
		ArithOp::And => Andi,
		ArithOp::Or => Ori,
		ArithOp::Xor => Xori,
		_ => unreachable!("float operation must use reg"),
	}
}

pub fn to_rop(op: &ArithOp) -> RTriInstrOp {
	match op {
		ArithOp::Add => Add,
		ArithOp::Sub => Sub,
		ArithOp::Mul => Mul,
		ArithOp::Div => Div,
		ArithOp::Rem => Rem,
		ArithOp::Shl => Sll,
		ArithOp::Lshr => Srl,
		ArithOp::Ashr => Sra,
		ArithOp::Fadd => Fadd,
		ArithOp::Fsub => Fsub,
		ArithOp::Fmul => Fmul,
		ArithOp::Fdiv => Fdiv,
		ArithOp::And => And,
		ArithOp::Or => Or,
		ArithOp::Xor => Xor,
	}
}
