// Ref：Engineering a Compiler 2nd Edition Page 433
mod compute_indvar;
mod helper_functions;
mod impls;
mod indvar_optimize;
mod move_invariant;
use std::collections::{HashMap, HashSet};

use llvm::{LlvmInstr, LlvmTemp, LlvmTempManager};
use rrvm::{rrvm_loop::LoopPtr, LlvmCFG, LlvmNode};

use crate::metadata::FuncData;

use super::{indvar::IndVar, temp_graph::TempGraph};

#[allow(clippy::upper_case_acronyms)]
#[allow(unused)]
pub struct IndVarSolver<'a> {
	// dfs 过程中，访问到的次序
	dfsnum: HashMap<LlvmTemp, i32>,
	next_dfsnum: i32,
	visited: HashSet<LlvmTemp>,
	// Tarjan 算法计算强连通分量时，需要用到的值
	low: HashMap<LlvmTemp, i32>,
	stack: Vec<LlvmTemp>,
	in_stack: HashSet<LlvmTemp>,
	cur_loop: LoopPtr,
	loop_invariant: HashSet<LlvmTemp>,
	// 每个变量映射到它所在的 scc 的 header
	header_map: HashMap<LlvmTemp, LlvmTemp>,
	preheader: LlvmNode,
	variants: HashSet<LlvmTemp>,
	// 对于一个 scc, 只记录 header
	useful_variants: HashSet<LlvmTemp>,
	useless_variants: HashSet<LlvmTemp>,
	// 不记录 0 阶归纳变量
	indvars: HashMap<LlvmTemp, IndVar>,
	// 函数参数
	params: HashSet<LlvmTemp>,
	new_invariant_instr: HashMap<LlvmTemp, LlvmInstr>,
	// 此过程是否做出了优化
	pub flag: bool,
	pub temp_graph: &'a mut TempGraph,
	mgr: &'a mut LlvmTempManager,
	loop_map: &'a mut HashMap<i32, LoopPtr>,
	def_map: &'a mut HashMap<LlvmTemp, LlvmNode>,
	funcdata: &'a mut FuncData,
	cfg: &'a LlvmCFG,
}
