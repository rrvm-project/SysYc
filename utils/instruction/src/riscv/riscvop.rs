use llvm::llvmop::ArithOp;
use sysyc_derive::Fuyuki;

pub use BranInstrOp::*;
pub use IBinInstrOp::*;
pub use ITriInstrOp::*;
pub use NoArgInstrOp::*;
pub use RBinInstrOp::*;
pub use RTriInstrOp::*;

#[derive(Fuyuki, PartialEq, Eq)]
pub enum ITriInstrOp {
	Addi,

	Xori,
	Ori,
	Andi,
	Slli,
	Srli,
	Srai,

	Slti,
	Sltiu,
}

#[derive(Fuyuki, PartialEq, Eq)]
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

#[derive(Fuyuki, PartialEq, Eq)]
pub enum IBinInstrOp {
	Li,
	Lui,
	LD,
	LW,
	LWU,
	SB,
	SH,
	SW,
	SD,
}

#[derive(Fuyuki, PartialEq, Eq)]
pub enum RBinInstrOp {
	#[style("fcvt.s.w")]
	Int2Float,
	#[style("fcvt.w.s")]
	Float2Int,
}

#[derive(Fuyuki, PartialEq, Eq)]
pub enum UnInstrOp {
	Li,
	Lb,
	Lh,
}

#[derive(Fuyuki, PartialEq, Eq)]
pub enum BranInstrOp {
	#[style("BEQ")]
	BEQ,
	#[style("BNE")]
	BNE,
	#[style("BLT")]
	BLT,
	#[style("BGE")]
	BGE,
	#[style("BLTU")]
	BLTU,
	#[style("BGEU")]
	BGEU,
}

#[derive(Fuyuki, PartialEq, Eq)]
pub enum NoArgInstrOp {
	Ret,
}

pub fn can_to_iop(op: &ArithOp) -> bool {
	matches!(
		op,
		ArithOp::Add
			| ArithOp::Shl
			| ArithOp::Lshr
			| ArithOp::Ashr
			| ArithOp::And
			| ArithOp::Or
			| ArithOp::Xor
	)
}

pub fn to_iop(op: &ArithOp) -> ITriInstrOp {
	match op {
		ArithOp::Add => Addi,
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
