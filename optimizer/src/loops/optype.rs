use llvm::{ArithOp, Value};

use super::OpType;

impl OpType {
	pub fn from_arithop(op: Option<ArithOp>, value: Value) -> Self {
		match op {
			Some(ArithOp::Add) => OpType::Add(value),
			Some(ArithOp::Fadd) => OpType::Fadd(value),
			Some(ArithOp::Sub) => OpType::Sub(value),
			Some(ArithOp::Fsub) => OpType::Fsub(value),
			Some(ArithOp::Mul) => OpType::Mul(value),
			Some(ArithOp::Fmul) => OpType::Fmul(value),
			Some(ArithOp::Div) => OpType::Div(value),
			Some(ArithOp::Fdiv) => OpType::Fdiv(value),
			Some(ArithOp::Rem) => OpType::Mod(value),
			_ => OpType::Others(value),
		}
	}
}
