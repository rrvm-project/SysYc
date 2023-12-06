use basicblock::Node;
use cfg::CFG;
use instruction::riscv::RiscvInstr;
use llvm::LlvmInstr;

pub mod basicblock;
pub mod cfg;
pub mod func;
pub mod impls;
pub mod program;

pub type LlvmCFG = CFG<LlvmInstr, llvm::Temp>;
pub type RiscvCFG = CFG<RiscvInstr, instruction::temp::Temp>;

pub type LlvmNode = Node<LlvmInstr, llvm::Temp>;
pub type RiscvNode = Node<RiscvInstr, instruction::temp::Temp>;
