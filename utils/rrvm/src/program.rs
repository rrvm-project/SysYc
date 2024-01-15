use std::any::Any;

use instruction::{riscv::RiscvInstr, temp};
use llvm::*;
use utils::{GlobalVar, InstrTrait, TempTrait};

use crate::func::RrvmFunc;

pub type LlvmFunc = RrvmFunc<LlvmInstr, LlvmTemp>;
pub type LlvmProgram = RrvmProgram<LlvmInstr, LlvmTemp, LlvmTempManager>;
pub type RiscvFunc = RrvmFunc<RiscvInstr, temp::Temp>;
pub type RiscvProgram = RrvmProgram<RiscvInstr, temp::Temp, temp::TempManager>;

pub struct RrvmProgram<T: InstrTrait<U>, U: TempTrait, M: Any> {
	pub global_vars: Vec<GlobalVar>,
	pub funcs: Vec<RrvmFunc<T, U>>,
	pub temp_mgr: M,
}
