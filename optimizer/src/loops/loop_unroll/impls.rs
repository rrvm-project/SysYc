use llvm::LlvmTempManager;
use rrvm::{program::LlvmFunc, rrvm_loop::LoopPtr};

use crate::{loops::loop_data::LoopData, metadata::FuncData};

use super::LoopUnroll;

impl<'a> LoopUnroll<'a> {
	pub fn new(
		func: &'a mut LlvmFunc,
		loopdata: &'a mut LoopData,
		funcdata: &'a mut FuncData,
		temp_mgr: &'a mut LlvmTempManager,
	) -> Self {
		Self {
			func,
			loopdata,
			funcdata,
			temp_mgr,
			flag: false,
		}
	}
	// 条件：只有一个 exit,指令总量小于 MAX_UNROLL_CNT,并且无内层循环
	pub fn apply(&mut self) -> bool {
		self.dfs(self.loopdata.root_loop.clone());
		self.flag
	}
	pub fn dfs(&mut self, loop_: LoopPtr) {
		let mut subloops = loop_.borrow().subloops.clone();
		subloops.retain(|subloop| {
			if subloop.borrow().no_inner() {
				if let Some(info) =
					self.loopdata.loop_infos.get(&subloop.borrow().id).cloned()
				{
					!self.unroll_one_loop(subloop.clone(), info)
				} else {
					true
				}
			} else {
				self.dfs(subloop.clone());
				true
			}
		});
		loop_.borrow_mut().subloops = subloops;
	}
}
