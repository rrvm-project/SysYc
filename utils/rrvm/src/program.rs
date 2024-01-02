use instruction::{riscv::RiscvInstr, temp};
use llvm::LlvmInstr;
use utils::{GlobalVar, InstrTrait, TempTrait};

use crate::func::RrvmFunc;

pub type LlvmFunc = RrvmFunc<LlvmInstr, llvm::Temp>;
pub type LlvmProgram = RrvmProgram<LlvmInstr, llvm::Temp>;
pub type RiscvFunc = RrvmFunc<RiscvInstr, temp::Temp>;
pub type RiscvProgram = RrvmProgram<RiscvInstr, temp::Temp>;

pub struct RrvmProgram<T: InstrTrait<U>, U: TempTrait> {
	pub global_vars: Vec<GlobalVar>,
	pub funcs: Vec<RrvmFunc<T, U>>,
	pub next_temp: u32,
}
