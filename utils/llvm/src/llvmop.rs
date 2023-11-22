use std::fmt::Display;

use sysyc_derive::Fuyuki;

use crate::{llvmvar::VarType, temp::Temp};

#[derive(Debug, Clone)]
pub enum Value {
	Int(i32),
	Float(f32),
	Temp(Temp),
}

pub trait LlvmOp: Display {
	fn oprand_type(&self) -> VarType;
}

#[derive(Fuyuki, Clone, Copy)]
pub enum ArithOp {
	Add,
	Sub,
	Div,
	Mul,
	Rem,  // modulo
	Fadd, // Float add
	Fsub, // Float sub
	Fdiv, // Float div
	Fmul, // Float mul
	Shl,
	Lshr, // logical shift right
	Ashr, // arithmetic shift right
	And,
	Or,
	Xor,
}

#[derive(Fuyuki)]
pub enum CompOp {
	EQ,
	NE,
	SGT, // signed greater than
	SGE, // signed greater or equal
	SLT, // signed less than
	SLE, // signed less or equal
	OEQ, // ordered and equal
	ONE, // ordered and not equal
	OGT, // ordered and greater than
	OGE, // ordered and greater or equal
	OLT, // ordered and less than
	OLE, // ordered and less or equal
}

pub fn is_commutative(op: &ArithOp) -> bool {
	matches!(
		op,
		ArithOp::Add
			| ArithOp::Mul
			| ArithOp::And
			| ArithOp::Or
			| ArithOp::Xor
			| ArithOp::Fadd
			| ArithOp::Fmul
	)
}

#[derive(Fuyuki)]
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
			Self::Temp(v) => v.var_type,
		}
	}
	pub fn is_num(&self) -> bool {
		!matches!(self, Self::Temp(_))
	}
	pub fn is_ptr(&self) -> bool {
		matches!(self, Self::Temp(v) if v.is_ptr())
	}
}

impl Display for Value {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Self::Int(v) => write!(f, "{}", v),
			Self::Float(v) => write!(f, "{}", v),
			Self::Temp(v) => write!(f, "{}", v),
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
