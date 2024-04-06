use llvm::LlvmTempManager;
use rrvm::{
	program::LlvmFunc,
	rrvm_loop::{utils::insert_preheader_for_loop, LoopPtr},
};

#[allow(unused)]
pub fn simplify_one_loop(
	func: &mut LlvmFunc,
	loop_: LoopPtr,
	temp_mgr: &mut LlvmTempManager,
) {
	// Does the loop already have a preheader?  If so, don't insert one.
	let preheader = match loop_
		.borrow()
		.get_loop_preheader()
		.or_else(|| insert_preheader_for_loop(loop_.clone(), func, temp_mgr))
	{
		Some(preheader) => preheader,
		None => return,
	};

	// Next, check to make sure that all exit nodes of the loop only have
	// predecessors that are inside of the loop.  This check guarantees that the
	// loop preheader/header will dominate the exit blocks.  If the exit block has
	// predecessors from outside of the loop, split the edge now.
}
