use llvm::{ArithOp, LlvmTemp, Value};

pub struct ChainNode {
	pub temp: LlvmTemp,
	pub op: ArithOp,
	pub operand: Vec<Value>,
}

impl ChainNode {
	pub fn new(temp: LlvmTemp, op: ArithOp, operand: Vec<Value>) -> Self {
		Self { temp, op, operand }
	}
}
