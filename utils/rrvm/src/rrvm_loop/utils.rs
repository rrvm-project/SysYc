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
	if let Some(preheader) =
		split_block_predecessors(header_rc, outside_blocks, func, temp_mgr)
	{
		// Update loop content
		loop_.borrow_mut().blocks.push(preheader.clone());
		Some(preheader)
	} else {
		None
	}
}

pub fn form_dedicated_exit_blocks(
	loop_: LoopPtr,
	func: &mut LlvmFunc,
	temp_mgr: &mut LlvmTempManager,
) {
	let mut rewrite_exit = |exit: LlvmNode| {
		let mut is_dedicated_exit = true;
		let mut in_loop_prev = Vec::new();
		for prev in exit.borrow().prev.iter() {
			if loop_.borrow().contains_block(prev) {
				if prev.borrow().succ.len() > 1 {
					return;
				}
				in_loop_prev.push(prev.clone());
			} else {
				is_dedicated_exit = false;
			}
		}
		assert!(!in_loop_prev.is_empty());

		if is_dedicated_exit {
			trace!("Already dedicated exit {}", exit.borrow().label());
			return;
		}

		let ret = split_block_predecessors(exit, in_loop_prev, func, temp_mgr);
		trace!(
			"Generated Dedicated exit {:?}",
			ret.map(|v| v.borrow().label())
		);
	};

	let mut visited = Vec::new();
	for bb in loop_.borrow().blocks.iter() {
		for succ in bb.borrow().succ.iter() {
			if !loop_.borrow().contains_block(succ) && !visited.contains(succ) {
				visited.push(succ.clone());
				trace!("Rewriting exit {:?}", succ.borrow().label());
				rewrite_exit(succ.clone());
			}
		}
	}
}

pub fn insert_unique_backedge_block(
	loop_: LoopPtr,
	func: &mut LlvmFunc,
	temp_mgr: &mut LlvmTempManager,
	preheader: LlvmNode,
) -> Option<LlvmNode> {
	let mut backedge_blocks = Vec::new();
	for prev in loop_.borrow().header.borrow().prev.iter() {
		if prev.borrow().succ.len() != 1 {
			trace!("Backedge has multiple successors");
			return None;
		}
		if *prev != preheader {
			backedge_blocks.push(prev.clone());
		}
	}

	trace!("Inserting unique backedge for loop");
	let header = loop_.borrow().header.clone();
	if let Some(backedge) =
		split_block_predecessors(header, backedge_blocks, func, temp_mgr)
	{
		// Update loop content
		loop_.borrow_mut().blocks.push(backedge.clone());
		Some(preheader)
	} else {
		None
	}
}
