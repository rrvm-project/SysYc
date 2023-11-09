use std::fmt::Display;

use crate::{llvmvar::VarType, temp::Temp};

pub enum Value {
	Int(i32),
	Float(f32),
	Temp(Temp),
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

impl Display for Value {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Self::Int(v) => write!(f, "{}", v),
			Self::Float(v) => write!(f, "{}", v),
			Self::Temp(v) => write!(f, "{}", v),
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

impl ArithOp {
	pub fn oprand_type(&self) -> VarType {
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
