use crate::riscvop::*;

use std::fmt::Display;
pub trait RiscvInstr: Display {
	fn get_write(&self) -> Value {
		Value::Register(RiscvReg::X0)
	}
	fn get_read(&self) -> Vec<Value> {
		Vec::new()
	}
}
