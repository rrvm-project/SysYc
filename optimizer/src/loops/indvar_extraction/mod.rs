use llvm::LlvmTempManager;
use rrvm::{dominator::LlvmDomTree, program::LlvmFunc};

use crate::metadata::FuncData;

use super::loop_data::LoopData;

mod impls;
mod one_loop_solver;

pub struct IndvarExtraction<'a> {
	pub loopdata: &'a mut LoopData,
	pub funcdata: &'a mut FuncData,
	pub temp_mgr: &'a mut LlvmTempManager,
	pub func: &'a mut LlvmFunc,
	pub dom_tree: LlvmDomTree,
}
