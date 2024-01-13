pub mod riscv;
pub mod temp;

use llvm::LlvmInstr;
use riscv::RiscvInstr;
pub use temp::Temp;

pub type LlvmInstrSet = Vec<LlvmInstr>;
pub type RiscvInstrSet = Vec<RiscvInstr>;
