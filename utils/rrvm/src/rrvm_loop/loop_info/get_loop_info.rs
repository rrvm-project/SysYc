// 识别 loop 中的信息

use crate::{rrvm_loop::LoopPtr, LlvmNode};

use super::SimpleLoopInfo;

// 粗略估计 call 指令会产生的指令数
const CALL_INSTR_NUM: i64 = 50;

#[allow(unused)]
pub fn get_loop_info(
	loop_: LoopPtr,
	loop_bbs: Vec<LlvmNode>,
	exit: LlvmNode,
	exit_prev: LlvmNode,
) -> SimpleLoopInfo {
	let mut info = SimpleLoopInfo::new();
	for block in loop_bbs.iter() {
		info.instr_cnt += block.borrow().phi_instrs.len() as i64;
		info.instr_cnt += block
			.borrow()
			.instrs
			.iter()
			.map(|i| if i.is_call() { CALL_INSTR_NUM } else { 1 })
			.sum::<i64>();
		info.instr_cnt += if block.borrow().jump_instr.is_some() {
			1
		} else {
			0
		};
	}
	let entry = loop_.borrow().header.clone();
	info.exit_prev = Some(exit_prev.clone());

	let mut into_entry = None;
	for prev in entry.borrow().prev.iter() {
		if prev.borrow().loop_.as_ref().is_some_and(|l| *l == loop_) {
			if *prev != exit_prev {
				return info; // 有多条回边，可能存在 continue
			}
		} else if into_entry.is_none() {
			into_entry = Some(prev.clone());
		} else {
			return info; // 有多个进入 entry 的块，这里可能可以尝试处理
		}
	}
	info.into_entry = into_entry;

	info
}
