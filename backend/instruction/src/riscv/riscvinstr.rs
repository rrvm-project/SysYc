use std::fmt::Display;

use super::{reg::RiscvReg, riscvop::*, value::Value};

pub trait RiscvInstr: Display {
	fn get_write(&self) -> Value {
		Value::Reg(RiscvReg::X0)
	}
	fn get_read(&self) -> Vec<Value> {
		Vec::new()
	}
}

pub struct RiscvTriInstr {
	pub op: TriInstrOp,
	pub target: Value,
	pub lhs: Value,
	pub rhs: Value,
}

// pub struct
