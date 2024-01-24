use llvm::{CompOp, Value};
use rrvm::{
	rrvm_loop::{
		loop_info::{get_loop_info::get_loop_info, LoopType},
		LoopPtr,
	},
	LlvmCFG, LlvmNode,
};

const UNROLL_CNT: usize = 4;

#[allow(unused)]
pub fn loop_unroll(cfg: &mut LlvmCFG, loop_: LoopPtr, func_params: &[Value]) {
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
	let loop_info = get_loop_info(
		cfg,
		func_params,
		loop_,
		loop_bbs,
		exit_bb.unwrap(),
		exit_prev.unwrap(),
	);

	println!("loop_info: \n{}", loop_info);

	if loop_info.instr_cnt > 100 {
		return;
	}
	if loop_info.loop_type == LoopType::IGNORE {
		return;
	}
	// 被展开次数
	let mut unroll_cnt = UNROLL_CNT;
	if loop_info.loop_type == LoopType::CONSTTERMINATED {
		// 总循环次数
		let mut full_cnt: i32;
		match loop_info.cond_op {
			CompOp::SLT => {
				full_cnt = (loop_info.end - loop_info.start + loop_info.step - 1)
					/ loop_info.step;
				if loop_info.start >= loop_info.end {
					full_cnt = 0;
				}
			}
			CompOp::SLE => {
				full_cnt =
					(loop_info.end - loop_info.start + loop_info.step) / loop_info.step;
				if loop_info.start > loop_info.end {
					full_cnt = 0;
				}
			}
			_ => unreachable!(),
		}
		if full_cnt <= 1 {
			return;
		}
		// 如果总循环次数比较小，或者该循环内指令的个数乘总循环次数比较小，就全部展开
		// 即，把循环体复制总循环次数次
		if (full_cnt < 350 || (full_cnt as i64) * loop_info.instr_cnt < 2000) {
			unroll_cnt = full_cnt as usize;
		}
	}
}
