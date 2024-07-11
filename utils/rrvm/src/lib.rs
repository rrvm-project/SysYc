pub mod basicblock;
pub mod cfg;
pub mod dominator;
pub mod func;
pub mod impls;
pub mod prelude;
pub mod program;

use basicblock::Node;
use cfg::CFG;
use instruction::riscv::RiscvInstr;
use llvm::LlvmInstr;

pub type LlvmCFG = CFG<LlvmInstr, llvm::LlvmTemp>;
pub type RiscvCFG = CFG<RiscvInstr, instruction::temp::Temp>;

pub type LlvmNode = Node<LlvmInstr, llvm::LlvmTemp>;
pub type RiscvNode = Node<RiscvInstr, instruction::temp::Temp>;
