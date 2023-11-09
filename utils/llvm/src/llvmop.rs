use std::fmt::Display;

use crate::{llvmvar::VarType, temp::Temp};

#[derive(Clone)]
pub enum Value {
	Int(i32),
	Float(f32),
	Temp(Temp),
	Void,
}

pub trait LlvmOp: Display {
	fn oprand_type(&self) -> VarType;
}

pub enum ArithOp {
	Add,
	Sub,
	Div,
	Mul,
	Rem,
	Fadd,
	Fsub,
	Fdiv,
	Fmul,
	Frem,
	Shl,
	Lshr,
	Ashr,
	And,
	Or,
	Xor,
}

pub enum CompOp {
	EQ,
	NE,
	SGT,
	SGE,
	SLT,
	SLE,
	OEQ,
	ONE,
	OGT,
	OGE,
	OLT,
	OLE,
}

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
			Self::Temp(v) => v.var_type.clone(),
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

impl Display for ArithOp {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let op_str = match self {
			Self::Add => "add",
			Self::Sub => "sub",
			Self::Div => "div",
			Self::Mul => "mul",
			Self::Rem => "rem",
			Self::Fadd => "fadd",
			Self::Fsub => "fsub",
			Self::Fdiv => "fdiv",
			Self::Fmul => "fmul",
			Self::Frem => "frem",
			Self::Shl => "shl",
			Self::Lshr => "lshr",
			Self::Ashr => "ashr",
			Self::And => "and",
			Self::Or => "or",
			Self::Xor => "xor",
		};
		write!(f, "{}", op_str)
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

impl Display for CompOp {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Self::EQ => write!(f, "eq"),
			Self::NE => write!(f, "ne"),
			Self::SGT => write!(f, "sgt"),
			Self::SGE => write!(f, "sge"),
			Self::SLT => write!(f, "slt"),
			Self::SLE => write!(f, "sle"),
			Self::OEQ => write!(f, "oeq"),
			Self::ONE => write!(f, "one"),
			Self::OGT => write!(f, "ogt"),
			Self::OGE => write!(f, "oge"),
			Self::OLT => write!(f, "olt"),
			Self::OLE => write!(f, "ole"),
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

impl Display for CompKind {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Self::Icmp => write!(f, "icmp"),
			Self::Fcmp => write!(f, "fcmp"),
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
			Self::Float2Int => write!(f, "fptpsi"),
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
