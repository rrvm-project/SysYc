mod classify_indvar;
mod get_loop_info;
mod helper_functions;
mod impls;
mod indvar_extraction;
mod tarjan_var;
mod utils;
use std::collections::{HashMap, HashSet};

use llvm::{LlvmInstr, LlvmTemp, LlvmTempManager};
use rrvm::{program::LlvmFunc, rrvm_loop::LoopPtr, LlvmNode};
use tarjan_var::TarjanVar;

use crate::{
	loops::{indvar::IndVar, loop_data::LoopData},
	metadata::FuncData,
};

#[allow(unused)]
// 认为循环内定义的变量都是循环变量，所有不变量已经被全部提出去了
pub struct OneLoopSolver<'a> {
	pub loopdata: &'a mut LoopData,
	pub funcdata: &'a mut FuncData,
	pub temp_mgr: &'a mut LlvmTempManager,
	pub func: &'a mut LlvmFunc,
	// tarjan 算法的变量
	tarjan_var: TarjanVar,
	cur_loop: LoopPtr,
	// 每个变量映射到它所在的 scc 的 header
	header_map: HashMap<LlvmTemp, LlvmTemp>,
	preheader: LlvmNode,
	// 对于一个 scc, 只记录 header
	useful_variants: HashSet<LlvmTemp>,
	// 不记录 0 阶归纳变量
	indvars: HashMap<LlvmTemp, IndVar>,
	new_invariant_instr: HashMap<LlvmTemp, LlvmInstr>,
	// 此过程是否做出了优化
	pub flag: bool,
}
