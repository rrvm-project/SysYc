use rrvm::{
	rrvm_loop::{loop_info::get_loop_info::get_loop_info, LoopPtr},
	LlvmNode,
};

#[allow(unused)]
pub fn loop_unroll(loop_: LoopPtr) {
	if !loop_.borrow().no_inner {
		return;
	}
	let mut loop_bbs: Vec<LlvmNode> = Vec::new();
	let mut stack = Vec::new();
	let mut insert_loop_bbs = |bb: LlvmNode| {
		if !loop_bbs.contains(&bb) {
			loop_bbs.push(bb);
		}
	};
	stack.push(loop_.borrow().header.clone());
	while let Some(stack_bb) = stack.pop() {
		if stack_bb.borrow().loop_.as_ref().is_some_and(|l| *l == loop_) {
			insert_loop_bbs(stack_bb.clone());
			stack.append(&mut stack_bb.borrow().dominates_directly.clone());
		}
	}
	// 确保循环只有一个 exit 和一个 exit_prev
	let mut exit_bb = None;
	let mut exit_prev = None;
	let mut check = true;
	for bb in loop_bbs.iter() {
		if !check {
			break;
		}
		for succ in bb.borrow().succ.iter() {
			if !succ.borrow().loop_.as_ref().is_some_and(|l| *l == loop_) {
				if exit_bb.as_ref().is_some() {
					check = false;
					break;
				}
				exit_bb = Some(succ.clone());
				exit_prev = Some(bb.clone());
			}
		}
	}
	if exit_bb.is_none() || !check {
		return;
	}
	let loop_info =
		get_loop_info(loop_, loop_bbs, exit_bb.unwrap(), exit_prev.unwrap());
	todo!()
}
