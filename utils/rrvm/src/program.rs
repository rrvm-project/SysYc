use instruction::{riscv::RiscvInstr, temp};
use llvm::LlvmInstr;
use utils::{InstrTrait, TempTrait};

use crate::func::RrvmFunc;

pub type LlvmFunc = RrvmFunc<LlvmInstr, llvm::Temp>;
pub type LlvmProgram = RrvmProgram<LlvmInstr, llvm::Temp>;
pub type RiscvFunc = RrvmFunc<RiscvInstr, temp::Temp>;
pub type RiscvProgram = RrvmProgram<RiscvInstr, temp::Temp>;

pub struct RrvmProgram<T: InstrTrait<U>, U: TempTrait> {
	// pub global_vars: Vec<>
	pub funcs: Vec<RrvmFunc<T, U>>,
}

impl<T: InstrTrait<U>, U: TempTrait> RrvmProgram<T, U> {
	pub fn new() -> Self {
		Self { funcs: Vec::new() }
	}
}

impl LlvmProgram {
	pub fn analysis(&mut self) {
		for func in self.funcs.iter() {
			func.cfg.init_phi();
			func.cfg.analysis();
		}
	}
}

impl<T: InstrTrait<U>, U: TempTrait> Default for RrvmProgram<T, U> {
	fn default() -> Self {
		Self::new()
	}
}
