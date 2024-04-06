use llvm::LlvmTempManager;
use rrvm::{
	program::LlvmFunc,
	rrvm_loop::{
		utils::{
			form_dedicated_exit_blocks, insert_preheader_for_loop,
			insert_unique_backedge_block,
		},
		LoopPtr,
	},
};

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
	form_dedicated_exit_blocks(loop_.clone(), func, temp_mgr);

	// If the header has more than two predecessors at this point (from the
	// preheader and from multiple backedges), we must adjust the loop.
	if loop_.borrow().get_loop_latch().is_none() {
		// I didn't seperate nested loops

		insert_unique_backedge_block(loop_, func, temp_mgr, preheader);
	}

	// If this loop has multiple exits and the exits all go to the same
	// block, attempt to merge the exits. This helps several passes, such
	// as LoopRotation, which do not support loops with multiple exits.
	// SimplifyCFG also does this (and this code uses the same utility
	// function), however this code is loop-aware, where SimplifyCFG is
	// not. That gives it the advantage of being able to hoist
	// loop-invariant instructions out of the way to open up more
	// opportunities, and the disadvantage of having the responsibility
	// to preserve dominator information.
}
