mod dominator_frontier;
mod impls;
mod naive;

use std::collections::HashMap;

pub use dominator_frontier::*;
use instruction::riscv::RiscvInstr;
use llvm::LlvmInstr;
pub use naive::*;
use utils::{InstrTrait, TempTrait};

use crate::cfg::Node;

pub type LlvmDomTree = DomTree<LlvmInstr, llvm::LlvmTemp>;
pub type RiscvDomTree = DomTree<RiscvInstr, instruction::temp::Temp>;

pub struct DomTree<T: InstrTrait<U>, U: TempTrait> {
	pub dominates: HashMap<i32, Vec<Node<T, U>>>,
	pub dominator: HashMap<i32, Node<T, U>>,
	pub dom_direct: HashMap<i32, Vec<Node<T, U>>>,
	pub df: HashMap<i32, Vec<Node<T, U>>>,
}
