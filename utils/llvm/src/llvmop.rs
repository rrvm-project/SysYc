use crate::{llvmvar::VarType, temp::Temp};
use serde_derive::Serialize;
use std::fmt::Display;

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
#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
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
#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
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
#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
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

impl Display for ArithOp {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		f.write_str(&serde_json::to_string(self).unwrap().trim_matches('\"'));
		Ok(())
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
		f.write_str(&serde_json::to_string(self).unwrap().trim_matches('\"'));
		Ok(())
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
		f.write_str(&serde_json::to_string(self).unwrap().trim_matches('\"'));
		Ok(())
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
