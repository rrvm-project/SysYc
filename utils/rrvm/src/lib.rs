use cfg::CFG;
use instruction::riscv::riscvinstr::RiscvInstr;
use llvm::LlvmInstr;

pub mod basicblock;
pub mod cfg;
pub mod func;
pub mod impls;
pub mod program;

pub type LlvmCFG = CFG<LlvmInstr>;
pub type RiscvCFG = CFG<RiscvInstr>;
