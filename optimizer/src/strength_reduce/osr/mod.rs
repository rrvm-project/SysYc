// Ref：Engineering a Compiler 2nd Edition Page 433
mod helper_functions;
mod impls;
mod lstf;
use std::collections::HashMap;

use llvm::{ArithOp, HashableValue, LlvmTemp};
use rrvm::LlvmNode;

use self::lstf::LSTFEdge;

#[allow(clippy::upper_case_acronyms)]
pub struct OSR {
	// dfs 过程中，访问到的次序
	dfsnum: HashMap<LlvmTemp, i32>,
	next_dfsnum: i32,
	visited: HashMap<LlvmTemp, bool>,
	// Tarjan 算法计算强连通分量时，需要用到的值
	low: HashMap<LlvmTemp, i32>,
	stack: Vec<LlvmTemp>,
	header: HashMap<LlvmTemp, LlvmTemp>,
	// 临时变量到（基本块id，基本块数组下标，指令数组下标，是否是 phi 指令）的映射
	temp_to_instr: HashMap<LlvmTemp, (i32, usize, usize, bool)>,

	// 记录因为候选操作而产生的指令，防止产生重复的指令
	new_instr: HashMap<(ArithOp, HashableValue, HashableValue), LlvmTemp>,
	// 此过程是否做出了优化
	pub flag: bool,

	dominates: HashMap<i32, Vec<LlvmNode>>,
	params: Vec<LlvmTemp>,
	lstf_map: HashMap<LlvmTemp, LSTFEdge>,
}
