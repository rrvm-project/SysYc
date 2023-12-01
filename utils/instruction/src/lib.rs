pub mod riscv;
pub mod temp;

use llvm::llvminstr::LlvmInstr;
use riscv::riscvinstr::RiscvInstr;

pub type LlvmInstrSet = Vec<LlvmInstr>;
pub type RiscvInstrSet = Vec<RiscvInstr>;
