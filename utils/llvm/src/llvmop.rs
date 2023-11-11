use std::fmt::Display;

use sysyc_derive::FuyukiDisplay;

use crate::{llvmvar::VarType, temp::Temp};

use sysyc_derive::FuyukiDisplay;

use crate::{llvmvar::VarType, temp::Temp};

#[derive(Clone, Debug)]
pub enum Value {
	Int(i32),
	Float(f32),
	Temp(Temp),
	Void,
}

pub trait LlvmOp: Display {
	fn oprand_type(&self) -> VarType;
}

#[derive(FuyukiDisplay)]
pub enum ArithOp {
	Add,
	Sub,
	Div,
	Mul,
	// modulo
	Rem,
	// Float add
	Fadd,
	// Float sub
	Fsub,
	// Float div
	Fdiv,
	// Float mul
	Fmul,
	// Float modulo
	Frem,
	// shift left
	Shl,
	// logical shift right
	Lshr,
	// arithmetic shift right
	Ashr,
	And,
	Or,
	Xor,
}

#[derive(FuyukiDisplay)]
pub enum CompOp {
	EQ,
	NE,
	// signed greater than
	SGT,
	// signed greater or equal
	SGE,
	// signed less than
	SLT,
	// signed less or equal
	SLE,
	// ordered and equal
	OEQ,
	// ordered and not equal
	ONE,
	// ordered and greater than
	OGT,
	// ordered and greater or equal
	OGE,
	// ordered and less than
	OLT,
	// ordered and less or equal
	OLE,
}

#[derive(FuyukiDisplay)]
pub enum CompKind {
	Icmp,
	Fcmp,
}

pub enum ConvertOp {
	Int2Float,
	Float2Int,
}

impl Value {
	pub fn get_type(&self) -> VarType {
		match self {
			Self::Int(_) => VarType::I32,
			Self::Float(_) => VarType::F32,
			Self::Void => VarType::Void,
			Self::Temp(v) => v.var_type,
		}
	}
}

impl Display for Value {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Self::Int(v) => write!(f, "{}", v),
			Self::Float(v) => write!(f, "{}", v),
			Self::Temp(v) => write!(f, "{}", v),
			Self::Void => write!(f, "void"),
		}
	}
}

impl LlvmOp for ArithOp {
	fn oprand_type(&self) -> VarType {
		match self {
			Self::Add => VarType::I32,
			Self::Sub => VarType::I32,
			Self::Div => VarType::I32,
			Self::Mul => VarType::I32,
			Self::Rem => VarType::I32,
			Self::Fadd => VarType::F32,
			Self::Fsub => VarType::F32,
			Self::Fdiv => VarType::F32,
			Self::Fmul => VarType::F32,
			Self::Frem => VarType::F32,
			Self::Shl => VarType::I32,
			Self::Lshr => VarType::I32,
			Self::Ashr => VarType::I32,
			Self::And => VarType::I32,
			Self::Or => VarType::I32,
			Self::Xor => VarType::I32,
		}
	}
}

impl LlvmOp for CompOp {
	fn oprand_type(&self) -> VarType {
		match self {
			Self::EQ => VarType::I32,
			Self::NE => VarType::I32,
			Self::SGT => VarType::I32,
			Self::SGE => VarType::I32,
			Self::SLT => VarType::I32,
			Self::SLE => VarType::I32,
			Self::OEQ => VarType::F32,
			Self::ONE => VarType::F32,
			Self::OGT => VarType::F32,
			Self::OGE => VarType::F32,
			Self::OLT => VarType::F32,
			Self::OLE => VarType::F32,
		}
	}
}

impl LlvmOp for CompKind {
	fn oprand_type(&self) -> VarType {
		match self {
			Self::Icmp => VarType::I32,
			Self::Fcmp => VarType::F32,
		}
	}
}

impl Display for ConvertOp {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Self::Int2Float => write!(f, "sitofp"),
			Self::Float2Int => write!(f, "fptosi"),
		}
	}
}

impl ConvertOp {
	pub fn type_from(&self) -> VarType {
		match self {
			Self::Float2Int => VarType::F32,
			Self::Int2Float => VarType::I32,
		}
	}
	pub fn type_to(&self) -> VarType {
		match self {
			Self::Float2Int => VarType::I32,
			Self::Int2Float => VarType::F32,
		}
	}
}
