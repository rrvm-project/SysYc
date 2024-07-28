use llvm::{ArithOp, Value};

use super::OpType;

impl OpType {
	pub fn from_arithop(op: Option<ArithOp>, value: Value) -> Self {
		match op {
			Some(ArithOp::Add) => OpType::Add(value),
			Some(ArithOp::Sub) => OpType::Sub(value),
			Some(ArithOp::Mul) => OpType::Mul(value),
			Some(ArithOp::Div) => OpType::Div(value),
			Some(ArithOp::Rem) => OpType::Mod(value),
			_ => OpType::Others(value),
		}
	}
}
