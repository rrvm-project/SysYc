use std::fmt::Display;

use instruction::riscv::riscvinstr::RiscvInstr;
use llvm::LlvmInstr;

use crate::func::RrvmFunc;

pub type LlvmFunc = RrvmFunc<LlvmInstr>;
pub type LlvmProgram = RrvmProgram<LlvmInstr>;
pub type RiscvProgram = RrvmProgram<RiscvInstr>;

pub struct RrvmProgram<T: Display> {
	// pub global_vars: Vec<>
	pub funcs: Vec<RrvmFunc<T>>,
}

impl<T: Display> RrvmProgram<T> {
	pub fn new() -> Self {
		Self { funcs: Vec::new() }
	}
}

impl<T: Display> Default for RrvmProgram<T> {
	fn default() -> Self {
		Self::new()
	}
}
