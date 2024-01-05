use std::{
	fmt::Display,
	hash::{Hash, Hasher},
};

use sysyc_derive::Fuyuki;

use crate::{llvmvar::VarType, temp::Temp};

use utils::{errors::Result, SysycError::*};

#[derive(Clone, Debug)]
pub enum Value {
	Int(i32),
	Float(f32),
	Temp(Temp),
}

impl Value {
	pub fn to_int(&self) -> Result<i32> {
		match self {
			Value::Int(v) => Ok(*v),
			_ => Err(SystemError("".to_string())),
		}
	}

	pub fn to_float(&self) -> Result<f32> {
		match self {
			Value::Float(v) => Ok(*v),
			_ => Err(SystemError("".to_string())),
		}
	}
}

impl PartialEq for Value {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::Int(l0), Self::Int(r0)) => l0 == r0,
			(Self::Float(l0), Self::Float(r0)) => l0 == r0,
			(Self::Temp(l0), Self::Temp(r0)) => l0 == r0,
			_ => false,
		}
	}
}

impl Hash for Value {
	fn hash<H: Hasher>(&self, state: &mut H) {
		core::mem::discriminant(self).hash(state);
		match self {
			Value::Int(i) => {
				i.hash(state);
			}
			Value::Float(f) => {
				let mut value = *f;
				if value.is_nan() || value.is_infinite() {
					value = 1926.0817f32;
				}
				value.to_bits().hash(state);
			}
			Value::Temp(t) => {
				t.hash(state);
			}
		}
	}
}

pub trait LlvmOp: Display {
	fn oprand_type(&self) -> VarType;
}

#[derive(Fuyuki, Clone, Copy, Debug, PartialEq, Hash, Eq)]
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
	AddD,
}

#[derive(Fuyuki, Clone, Copy, PartialEq, Hash)]
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

#[derive(Fuyuki, Clone, Copy)]
pub enum CompKind {
	Icmp,
	Fcmp,
}

#[derive(Clone, Copy)]
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
	pub fn deref_type(&self) -> VarType {
		match self {
			Self::Temp(v) => v.var_type.deref_type(),
			_ => unreachable!(),
		}
	}
	pub fn is_num(&self) -> bool {
		!matches!(self, Self::Temp(_))
	}
	pub fn always_true(&self) -> bool {
		match self {
			Self::Int(v) => *v != 0,
			_ => false,
		}
	}
	pub fn always_false(&self) -> bool {
		match self {
			Self::Int(v) => *v == 0,
			_ => false,
		}
	}
	pub fn is_global(&self) -> bool {
		matches!(self, Self::Temp(v) if v.is_global)
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
			_ => unreachable!(),
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
