use llvm::{ArithOp, LlvmTemp, Value};

pub struct ChainNode {
	pub temp: LlvmTemp,
	pub op: ArithOp,
	pub operand: Value,
}

impl ChainNode {
	pub fn new(temp: LlvmTemp, op: ArithOp, operand: Value) -> Self {
		Self { temp, op, operand }
	}
}
