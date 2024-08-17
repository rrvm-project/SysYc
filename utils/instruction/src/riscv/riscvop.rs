use llvm::ArithOp;
use sysyc_derive::Fuyuki;

pub use BranInstrOp::*;
pub use IBinInstrOp::*;
pub use ITriInstrOp::*;
pub use NoArgInstrOp::*;
pub use RBinInstrOp::*;
pub use RTriInstrOp::*;
pub use TemporayInstrOp::*;

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
	Addw,
	Sub,
	Subw,
	Mul,
	Mulw,
	Rem,
	Remw,
	Div,
	Divw,
	Xor,
	Xorw,
	Or,
	Orw,
	And,
	Andw,
	Sll,
	Sllw,
	Srl,
	Srlw,
	Sra,
	Sraw,

	#[style("clz.d")]
	Clz,
	#[style("clz.w")]
	Clzw,
	#[style("ctz.d")]
	Ctz,
	#[style("ctz.w")]
	Ctzw,

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

	#[style("min.d")]
	Min,
	#[style("min.w")]
	Minw,
	#[style("max.d")]
	Max,
	#[style("max.w")]
	Maxw,
	#[style("sh1add.w")]
	Sh1addw,
	#[style("sh2add.w")]
	Sh2addw,
	#[style("sh3add.w")]
	Sh3addw,
	#[style("sh1add.d")]
	Sh1add,
	#[style("sh2add.d")]
	Sh2add,
	#[style("sh3add.d")]
	Sh3add,

	#[style("feq.s")]
	Feq,
	#[style("flt.s")]
	Flt,
	#[style("fle.s")]
	Fle,

	#[style("fabs.s")]
	Fabs,
	#[style("fneg.s")]
	Fneg,
	#[style("fmin.s")]
	Fmin,
	#[style("fmax.s")]
	Fmax,
}

#[derive(Fuyuki, PartialEq, Eq, Clone, Copy)]
pub enum IBinInstrOp {
	Li,
	LD,
	LW,
	LWU,
	SB,
	SH,
	SW,
	SD,
	LA,
	Auipc,
	// float
	FLW,
	FSW,
	FLD,
	FSD,
}

#[derive(Fuyuki, PartialEq, Eq, Clone, Copy)]
pub enum RBinInstrOp {
	Mv,
	#[style("fmv.s")]
	FMv,
	#[style("fmv.w.x")]
	MvInt2Float,
	#[style("fcvt.s.w")]
	Int2Float,
	#[style("fcvt.w.s")]
	Float2Int,
	#[style("sext.w")]
	Sextw,
	Seqz,
	Snez,
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

#[derive(Fuyuki, PartialEq, Eq, Clone, Copy, Debug)]
pub enum TemporayInstrOp {
	Save,
	Restore,
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
		ArithOp::AddD => Add,
		ArithOp::Sub => Subw,
		ArithOp::SubD => Sub,
		ArithOp::Mul => Mulw,
		ArithOp::MulD => Mul,
		ArithOp::Div => Divw,
		ArithOp::DivD => Div,
		ArithOp::Rem => Remw,
		ArithOp::RemD => Rem,
		ArithOp::Shl => Sllw,
		ArithOp::ShlD => Sll,
		ArithOp::Lshr => Srlw,
		ArithOp::LshrD => Srl,
		ArithOp::Ashr => Sraw,
		ArithOp::AshrD => Sra,
		ArithOp::And => And,
		ArithOp::Or => Or,
		ArithOp::Xor => Xor,
		ArithOp::Fadd => Fadd,
		ArithOp::Fsub => Fsub,
		ArithOp::Fmul => Fmul,
		ArithOp::Fdiv => Fdiv,
		ArithOp::Clz => Clzw,
		ArithOp::ClzD => Clz,
		ArithOp::Ctz => Ctzw,
		ArithOp::CtzD => Ctz,
		ArithOp::Min => Minw,
		ArithOp::MinD => Min,
		ArithOp::Max => Maxw,
		ArithOp::MaxD => Max,
		ArithOp::Fmin => Fmin,
		ArithOp::Fmax => Fmax,
	}
}
