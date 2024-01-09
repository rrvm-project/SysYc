use llvm::llvmop::ArithOp;
use sysyc_derive::Fuyuki;

pub use BranInstrOp::*;
pub use IBinInstrOp::*;
pub use ITriInstrOp::*;
pub use NoArgInstrOp::*;
pub use RBinInstrOp::*;
pub use RTriInstrOp::*;

#[derive(Fuyuki, PartialEq, Eq, Clone, Copy)]
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

	Addiw,
	Slliw,
	Srliw,
	Sraiw,
	Sltiw,
}

#[derive(Fuyuki, PartialEq, Eq, Clone, Copy)]
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

	Addw,
	Subw,
	Mulw,
	Remw,
	Divw,
	Sllw,
	Srlw,
	Sraw,

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

#[derive(Fuyuki, PartialEq, Eq, Clone, Copy)]
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
	LA,
}

#[derive(Fuyuki, PartialEq, Eq, Clone, Copy)]
pub enum RBinInstrOp {
	#[style("fcvt.s.w")]
	Int2Float,
	#[style("fcvt.w.s")]
	Float2Int,
	#[style("sext.w")]
	Sextw,
	Negw,
	Neg,
}

#[derive(Fuyuki, PartialEq, Eq, Clone, Copy)]
pub enum BranInstrOp {
	Beq,
	Bne,
	Blt,
	Bge,
	Bltu,
	Bgeu,
}

#[derive(Fuyuki, PartialEq, Eq, Clone, Copy)]
pub enum NoArgInstrOp {
	Ret,
}

pub fn can_to_iop(op: &ArithOp) -> bool {
	matches!(
		op,
		ArithOp::Add
			| ArithOp::AddD
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
		ArithOp::AddD => Addi,
		ArithOp::Add => Addiw,
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
		ArithOp::Add => Addw,
		ArithOp::Sub => Subw,
		ArithOp::Mul => Mulw,
		ArithOp::Div => Divw,
		ArithOp::Rem => Remw,
		ArithOp::Shl => Sllw,
		ArithOp::Lshr => Srlw,
		ArithOp::Ashr => Sraw,
		ArithOp::Fadd => Fadd,
		ArithOp::Fsub => Fsub,
		ArithOp::Fmul => Fmul,
		ArithOp::Fdiv => Fdiv,
		ArithOp::And => And,
		ArithOp::Or => Or,
		ArithOp::Xor => Xor,
		ArithOp::AddD => Add,
	}
}
