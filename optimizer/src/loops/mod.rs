use std::collections::HashMap;

use llvm::{LlvmTemp, Value};

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
}

pub struct TempGraph {
	// 从自己指向自己的 use
	temp_graph: HashMap<LlvmTemp, Vec<OpType>>,
}

pub enum OpType {
	Add(Value),
	Sub(Value),
	Mul(Value),
	Div(Value),
	// 取模
	Mod(Value),
	Phi(Value),
	// TODO：这里可能还可以扩展
	Others(Value),
}
