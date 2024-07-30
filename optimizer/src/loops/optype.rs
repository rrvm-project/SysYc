use llvm::{ArithOp, Value};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpType {
	Add(Value),
	Fadd(Value),
	Sub(Value),
	Fsub(Value),
	Mul(Value),
	Fmul(Value),
	Div(Value),
	Fdiv(Value),
	// 取模
	Mod(Value),
	Phi(Value),
	// TODO：这里可能还可以扩展
	Others(Value),
}

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
