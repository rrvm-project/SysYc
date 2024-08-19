use std::{
	collections::HashMap,
	fmt::Display,
	hash::{Hash, Hasher},
};

use sysyc_derive::Fuyuki;

use crate::{llvmvar::VarType, LlvmTemp};

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
	Int(i32),
	Float(f32),
	Temp(LlvmTemp),
}

impl Eq for Value {}

impl Hash for Value {
	fn hash<H: Hasher>(&self, state: &mut H) {
		core::mem::discriminant(self).hash(state);
		match self {
			Value::Int(i) => {
				i.hash(state);
			}
			Value::Float(f) => {
				f.to_bits().hash(state);
			}
			Value::Temp(t) => {
				t.hash(state);
			}
		}
	}
}
impl Value {
	pub fn is_zero(&self) -> bool {
		match self {
			Self::Int(v) => *v == 0,
			Self::Float(v) => *v == 0.0,
			_ => false,
		}
	}
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HashableValue {
	Int(i32),
	Float(u64, i16, i8),
	Temp(LlvmTemp),
}

#[derive(Fuyuki, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ArithOp {
	Add,
	AddD,
	Mul,
	MulD,
	Sub,
	SubD,
	Div,
	DivD,
	Rem, // modulo
	RemD,
	Shl,
	ShlD,
	Lshr, // logical shift right
	LshrD,
	Ashr, // arithmetic shift right
	AshrD,
	And,
	Or,
	Xor,
	Clz, // count leading zeros
	ClzD,
	Ctz, // count trailing zeros
	CtzD,
	Min, // min
	MinD,
	Max, // max
	MaxD,
	Fadd, // Float add
	Fsub, // Float sub
	Fdiv, // Float div
	Fmul, // Float mul
	Fmin, // Float min
	Fmax, // Float max
}

#[derive(Fuyuki, Clone, Copy, PartialEq, Eq, Hash, Debug)]
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
	OLE, // ordered and less or
}

impl ArithOp {
	pub fn is_commutative(&self) -> bool {
		matches!(
			self,
			ArithOp::Add
				| ArithOp::Mul
				| ArithOp::And
				| ArithOp::Or
				| ArithOp::Xor
				| ArithOp::Fadd
				| ArithOp::Fmul
		)
	}
	pub fn to_int_op(&self) -> Self {
		match self {
			Self::Fadd => Self::Add,
			Self::Fsub => Self::Sub,
			Self::Fdiv => Self::Div,
			Self::Fmul => Self::Mul,
			_ => *self,
		}
	}
	pub fn to_float_op(&self) -> Self {
		match self {
			Self::Add => Self::Fadd,
			Self::Sub => Self::Fsub,
			Self::Div => Self::Fdiv,
			Self::Mul => Self::Fmul,
			_ => *self,
		}
	}
}

pub fn is_commutative(op: &ArithOp) -> bool {
	matches!(
		op,
		ArithOp::Add
			| ArithOp::Mul
			| ArithOp::MulD
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

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
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
			Self::Float(v) => !v.is_nan() && !v.is_infinite() && *v != 0.0,
			_ => false,
		}
	}
	pub fn always_false(&self) -> bool {
		match self {
			Self::Int(v) => *v == 0,
			Self::Float(v) => v.is_nan() || v.is_infinite() || *v == 0.0,
			_ => false,
		}
	}
	pub fn is_global(&self) -> bool {
		matches!(self, Self::Temp(v) if v.is_global)
	}
	pub fn unwrap_temp(&self) -> Option<LlvmTemp> {
		match self {
			Self::Temp(v) => Some(v.clone()),
			_ => None,
		}
	}
	pub fn map_temp(&mut self, mapper: &HashMap<LlvmTemp, Value>) {
		if let Some(temp) = self.unwrap_temp() {
			if let Some(value) = mapper.get(&temp) {
				*self = value.clone();
			}
		}
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
