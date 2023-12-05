use std::fmt::Display;

use instruction::{riscv::riscvinstr::RiscvInstr, temp};
use llvm::LlvmInstr;
use utils::UseTemp;

use crate::func::RrvmFunc;

pub type LlvmFunc = RrvmFunc<LlvmInstr, llvm::Temp>;
pub type LlvmProgram = RrvmProgram<LlvmInstr, llvm::Temp>;
pub type RiscvFunc = RrvmFunc<RiscvInstr, temp::Temp>;
pub type RiscvProgram = RrvmProgram<RiscvInstr, temp::Temp>;

pub struct RrvmProgram<T: Display + UseTemp<U>, U: Display> {
	// pub global_vars: Vec<>
	pub funcs: Vec<RrvmFunc<T, U>>,
}

impl<T: Display + UseTemp<U>, U: Display> RrvmProgram<T, U> {
	pub fn new() -> Self {
		Self { funcs: Vec::new() }
	}
}

impl<T: Display + UseTemp<U>, U: Display> Default for RrvmProgram<T, U> {
	fn default() -> Self {
		Self::new()
	}
}
