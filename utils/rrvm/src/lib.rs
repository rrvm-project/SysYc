pub mod basicblock;
pub mod cfg;
pub mod dominator;
pub mod func;
pub mod impls;
pub mod prelude;
pub mod program;
pub mod rrvm_loop;

use basicblock::Node;
use cfg::CFG;
use instruction::{riscv::RiscvInstr, Temp};
use llvm::LlvmInstr;

pub type LlvmCFG = CFG<LlvmInstr, llvm::LlvmTemp>;
pub type RiscvCFG = CFG<RiscvInstr, Temp>;

pub type LlvmNode = Node<LlvmInstr, llvm::LlvmTemp>;
pub type RiscvNode = Node<RiscvInstr, Temp>;
