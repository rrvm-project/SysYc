use std::collections::HashMap;

use llvm::{LlvmTemp, Value};
use rrvm::rrvm_loop::LoopPtr;

pub mod display;
pub mod impls;
pub mod indvar;
pub mod loop_optimizer;
pub mod loopinfo;
pub mod optype;
pub mod temp_graph;

pub struct HandleLoops {}

pub struct LoopOptimizer {
	// 从自己指向自己的 use
	temp_graph: TempGraph,
	// 每个 basicblock 属于哪个循环
	loop_map: HashMap<i32, LoopPtr>,
}

pub struct TempGraph {
	// 从自己指向自己的 use
	temp_graph: HashMap<LlvmTemp, Vec<OpType>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpType {
	Add(Value),
	Fadd(Value),
	Sub(Value),
	Fsub(Value),
	Mul(Value),
	Fmul(Value),
	Div(Value),
	Fdiv(Value),
	// 取模
	Mod(Value),
	Phi(Value),
	// TODO：这里可能还可以扩展
	Others(Value),
}