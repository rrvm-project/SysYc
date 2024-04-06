use llvm::LlvmTempManager;
use log::trace;

use crate::{
	prelude::{split_block_predecessors, LlvmFunc},
	LlvmNode,
};

use super::LoopPtr;

pub fn is_legal_to_hoist_into(bb: LlvmNode) -> bool {
	assert!(!bb.borrow().succ.is_empty());
	return !bb.borrow().jump_instr.as_ref().unwrap().is_ret();
}

/// InsertPreheaderForLoop - Once we discover that a loop doesn't have a
/// preheader, this method is called to insert one.  This method has two phases:
/// preheader insertion and analysis updating.
///
pub fn insert_preheader_for_loop(
	loop_: LoopPtr,
	func: &mut LlvmFunc,
	temp_mgr: &mut LlvmTempManager,
) -> Option<LlvmNode> {
	let header_rc = loop_.borrow().header.clone();
	let mut outside_blocks = Vec::new();
	for prev in header_rc.clone().borrow().prev.iter() {
		if !loop_.borrow().contains_block(prev) {
			// If the loop is branched to from an indirect branch, we won't
			// be able to fully transform the loop, because it prohibits
			// edge splitting.
			if prev.borrow().succ.len() != 1 {
				trace!(
					"Succ num is {}, cannot insert preheader",
					prev.borrow().succ.len()
				);
				return None;
			}
			outside_blocks.push(prev.clone());
		}
	}

	trace!("Inserting preheader for loop");
	split_block_predecessors(header_rc, outside_blocks, func, temp_mgr)
}

#[allow(unused)]
pub fn form_dedicated_exit_blocks(loop_: LoopPtr, func: &mut LlvmFunc) {}
