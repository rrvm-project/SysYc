use llvm::LlvmTempManager;
use rrvm::program::LlvmFunc;

use crate::metadata::FuncData;

use super::loop_data::LoopData;

pub mod impls;
pub mod loop_unroll;

pub struct LoopUnroll<'a> {
	pub loopdata: &'a mut LoopData,
	pub funcdata: &'a mut FuncData,
	pub temp_mgr: &'a mut LlvmTempManager,
	pub func: &'a mut LlvmFunc,
	pub flag: bool,
}
