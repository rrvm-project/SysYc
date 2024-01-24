// 识别 loop 中的信息

use llvm::{Value, VarType};

use crate::{rrvm_loop::LoopPtr, LlvmCFG, LlvmNode};

use super::{LoopType, SimpleLoopInfo};

// 粗略估计 call 指令会产生的指令数
const CALL_INSTR_NUM: i64 = 50;

#[allow(unused)]
pub fn get_loop_info(
	cfg: &mut LlvmCFG,
	func_params: &[Value],
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

	let type_ = LoopType::VARTEMINATED;

	if let Some(jump_instr) = exit_prev.borrow().jump_instr.as_ref() {
		if jump_instr.is_jump_cond() {
			let cond_temp = jump_instr.get_read().first().cloned().unwrap();
			let exit_prev_borrow = exit_prev.borrow();
			let def_cond_temp = exit_prev_borrow
				.instrs
				.iter()
				.find(|instr| instr.get_write().is_some_and(|w| w == cond_temp))
				.expect("jump cond temp not found");
			if def_cond_temp.is_loop_unroll_cond_op() {
				let (lhs, rhs) = def_cond_temp.get_lhs_and_rhs().unwrap();
				// if func_params.contains(&rhs) {
				// 	info.end_temp = Some(rhs.unwrap_temp().unwrap().clone());
				// } else
				if let Value::Int(int_value) = rhs {
					info.end = int_value;
				} else {
					info.end_temp = Some(rhs.unwrap_temp().unwrap().clone());
				}
				info.cond_op = def_cond_temp.get_comp_op().unwrap();
				info.cond_temp = Some(cond_temp);

				if func_params.contains(&lhs) {
					return info;
				}
				if lhs.is_num() {
					return info;
				}
				let lhs = lhs.unwrap_temp().unwrap();
				let def_lhs = exit_prev_borrow
					.instrs
					.iter()
					.find(|instr| instr.get_write().is_some_and(|w| w == lhs))
					.unwrap();
				if def_lhs.get_write().unwrap().var_type != VarType::I32 {
					return info;
				}
			}
		}
	}
	info
}
