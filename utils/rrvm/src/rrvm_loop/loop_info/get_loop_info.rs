// 识别 loop 中的信息

use std::collections::HashMap;

use llvm::{LlvmInstrTrait, LlvmTemp, Value};
use log::trace;
use utils::UseTemp;

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
	info.exit = Some(exit.clone());

	let mut into_entry = None;

	for prev in entry.borrow().prev.iter() {
		if prev.borrow().loop_.as_ref().is_some_and(|l| *l == loop_) {
			if info.backedge_start.is_none() {
				info.backedge_start = Some(prev.clone());
			} else {
				trace!("IGNORE: multiple backedge start");
				println!("IGNORE: multiple backedge start");
				return info; // 有多条回边，可能存在 continue
			}
		} else if into_entry.is_none() {
			into_entry = Some(prev.clone());
		} else {
			trace!("IGNORE: multiple into entry");
			println!("IGNORE: multiple into entry");
			return info; // 有多个进入 entry 的块，这里可能可以尝试处理
		}
	}
	info.into_entry = into_entry;

	let def_map = construct_def_map(cfg);

	let mut type_ = LoopType::VARTEMINATED;

	if let Some(jump_instr) = exit_prev.borrow().jump_instr.as_ref() {
		if jump_instr.is_jump_cond() {
			let cond_temp = jump_instr.get_read().first().cloned().unwrap();
			let exit_prev_borrow = exit_prev.borrow();
			// TODO：分支指令读取的临时变量的定义可能和分支指令不在同一个基本块内
			// let def_cond_temp = exit_prev_borrow
			// 	.instrs
			// 	.iter()
			// 	.find(|instr| instr.get_write().is_some_and(|w| w == cond_temp))
			// 	.expect("jump cond temp not found");
			let def_cond_temp =
				def_map.get(&cond_temp).expect("jump cond temp not found");
			if def_cond_temp.is_loop_unroll_cond_op() {
				// 只考虑 i < n 和 i <= n
				let (lhs, rhs) = def_cond_temp.get_lhs_and_rhs().unwrap();
				if func_params.contains(&rhs) {
					info.end_temp = Some(rhs.unwrap_temp().unwrap().clone());
				} else if let Value::Int(int_value) = rhs {
					info.end = int_value;
					type_ = LoopType::CONSTTERMINATED;
				} else {
					info.end_temp = Some(rhs.unwrap_temp().unwrap().clone());
				}
				info.cond_op = def_cond_temp.get_comp_op().unwrap();
				info.cond_temp = Some(cond_temp);

				if func_params.contains(&lhs) {
					trace!("IGNORE: loop end condition lhs is a function parameter");
					println!("IGNORE: loop end condition lhs is a function parameter");
					return info;
				}
				if lhs.is_num() {
					trace!("IGNORE: loop end condition lhs is a constant");
					println!("IGNORE: loop end condition lhs is a constant");
					return info;
				}
				let lhs = lhs.unwrap_temp().unwrap();
				if let Some((start, update_temp, phi_temp, update)) =
					is_simple_induction_variable(lhs, &def_map)
				{
					if update != 1 {
						trace!("IGNORE: loop update is not 1, it is {}", update);
						println!("IGNORE: loop update is not 1, it is {}", update);
						return info;
					}
					info.step = update;
					info.start = start;
					info.indvar_temp = Some(update_temp);
					info.phi_temp = Some(phi_temp);
				} else {
					trace!(
						"IGNORE: loop end condition lhs is not a simple induction variable"
					);
					println!(
						"IGNORE: loop end condition lhs is not a simple induction variable"
					);
					return info;
				}
			}
		} else {
			panic!("jump instr of a loop's exit_prev is not jump cond");
		}
	}
	if type_ == LoopType::VARTEMINATED {
		trace!("IGNORE: variable terminated");
		println!("IGNORE: variable terminated");
		return info;
	}
	info.loop_type = type_;
	info
}

// 传入一个 %1，检查是否存在
// %1 = phi i32 [0, label %_], [%2, label %_]
// %2 = add i32 %1, 1
// 且它们在同一个基本块内
// 返回 (0, %2, %1, 1)
fn is_simple_induction_variable(
	temp: LlvmTemp,
	def_map: &HashMap<LlvmTemp, Box<dyn LlvmInstrTrait>>,
) -> Option<(i32, LlvmTemp, LlvmTemp, i32)> {
	let get_int_and_temp = |v: &[Value]| -> Option<(i32, LlvmTemp)> {
		if let Value::Int(i) = v[0] {
			if let Value::Temp(t) = &v[1] {
				return Some((i, t.clone()));
			}
		} else if let Value::Int(i) = v[1] {
			if let Value::Temp(t) = &v[0] {
				return Some((i, t.clone()));
			}
		}
		None
	};

	let def_temp = def_map.get(&temp)?;

	if def_temp.is_phi() {
		let read_values = def_temp.get_read_values();
		if read_values.len() != 2 {
			return None;
		}
		if let Some((i, t)) = get_int_and_temp(&read_values) {
			// let def_t = block
			// 	.instrs
			// 	.iter()
			// 	.find(|instr| instr.get_write().is_some_and(|w| w == t))?;
			let def_t = def_map.get(&t)?;

			if !def_t.is_loop_unroll_update_op() {
				return None;
			}
			let read_values = def_t.get_read_values();
			if read_values.len() != 2 {
				return None;
			}
			if let Some((i2, t2)) = get_int_and_temp(&read_values) {
				if t2 == temp {
					return Some((i, t, t2, i2));
				} else {
					return None;
				}
			}
		}
	} else {
		if !def_temp.is_loop_unroll_update_op() {
			return None;
		}
		let read_values = def_temp.get_read_values();
		if read_values.len() != 2 {
			return None;
		}
		if let Some((i, t)) = get_int_and_temp(&read_values) {
			// let def_t = block
			// 	.instrs
			// 	.iter()
			// 	.find(|instr| instr.get_write().is_some_and(|w| w == t))?;
			let def_t = def_map.get(&t)?;
			if !def_t.is_phi() {
				return None;
			}
			let read_values = def_t.get_read_values();
			if read_values.len() != 2 {
				return None;
			}
			if let Some((i2, t2)) = get_int_and_temp(&read_values) {
				if t2 == temp {
					return Some((i2, t2, t, i));
				} else {
					return None;
				}
			}
		}
	}
	None
}

fn construct_def_map(
	cfg: &LlvmCFG,
) -> HashMap<LlvmTemp, Box<dyn LlvmInstrTrait>> {
	let mut def_map = HashMap::new();
	for block in cfg.blocks.iter() {
		let block = block.borrow();
		for instr in block.instrs.iter() {
			if let Some(temp) = instr.get_write() {
				def_map.insert(temp, instr.clone());
			}
		}
		for instr in block.phi_instrs.iter() {
			if let Some(temp) = instr.get_write() {
				def_map.insert(temp, Box::new(instr.clone()));
			}
		}
	}
	def_map
}
