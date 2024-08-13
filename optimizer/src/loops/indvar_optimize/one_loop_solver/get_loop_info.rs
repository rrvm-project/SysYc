use llvm::{CompOp, LlvmInstrVariant, Value};

use crate::loops::loopinfo::LoopInfo;

use super::OneLoopSolver;

impl<'a> OneLoopSolver<'a> {
	// 如果不能确定循环总次数，则返回 None
	pub fn get_loop_info(&mut self) -> Option<LoopInfo> {
		let header = self.cur_loop.borrow().header.clone();
		let preheader = self.preheader.clone();
		let single_exit = match self.cur_loop.borrow().get_single_exit(
			&self
				.cur_loop
				.borrow()
				.blocks_without_subloops(&self.func.cfg, &self.loopdata.loop_map),
			&self.loopdata.loop_map,
		) {
			Some(bb) => bb,
			_ => return None,
		};

		// 取非
		let mut take_not = false;
		let header_borrowed = header.borrow();
		if let Some(jump_instr) = header_borrowed.jump_instr.as_ref() {
			match jump_instr.get_variant() {
				LlvmInstrVariant::JumpCondInstr(cond_inst) => {
					// 默认条件不成立跳出循环，并且默认 lhs 是 indvar
					// 如果是条件成立就跳出循环，相当于条件取非后，条件不成立跳出循环
					if cond_inst.target_true == single_exit.borrow().label() {
						take_not = true;
					}
					let branch_temp = jump_instr.get_read().first().cloned().unwrap();
					let def_branch_temp =
						self.loopdata.temp_graph.temp_to_instr[&branch_temp].instr.clone();
					match def_branch_temp.get_variant() {
						LlvmInstrVariant::CompInstr(inst) => {
							if matches!(
								inst.op,
								CompOp::SLT | CompOp::SLE | CompOp::SGT | CompOp::SGE
							) {
								let get_info = |cond_value: Value,
								                end_value: Value,
								                take_reverse: bool|
								 -> Option<LoopInfo> {
									if self.is_loop_invariant(&end_value) {
										if let Some(t) = cond_value.unwrap_temp() {
											if let Some(iv) = self.indvars.get(&t).cloned() {
												if iv.step.len() > 1 {
													return None;
												}
												let new_op =
													convert_comp_op(inst.op, take_not, take_reverse);
												#[cfg(feature = "debug")]
												eprintln!("get loop info: Found a loop to optimize with cond_temp: {} {}", t, iv);
												let info = LoopInfo {
													preheader: preheader.clone(),
													header: header.clone(),
													single_exit: single_exit.clone(),
													cmp: branch_temp.clone(),
													comp_op: new_op,
													step: iv.step[0].clone(),
													begin: iv.base,
													end: end_value,
												};
												return Some(info);
											}
										}
									}
									None
								};
								get_info(inst.lhs.clone(), inst.rhs.clone(), false).or_else(
									|| get_info(inst.rhs.clone(), inst.lhs.clone(), true),
								)
							} else {
								None
							}
						}
						_ => None,
					}
				}
				_ => None,
			}
		} else {
			None
		}
	}
}

fn convert_comp_op(
	mut op: CompOp,
	take_not: bool,
	take_reverse: bool,
) -> CompOp {
	assert!(matches!(
		op,
		CompOp::SLT | CompOp::SLE | CompOp::SGT | CompOp::SGE
	));
	op = if take_not {
		match op {
			CompOp::SLT => CompOp::SGE,
			CompOp::SLE => CompOp::SGT,
			CompOp::SGT => CompOp::SLE,
			CompOp::SGE => CompOp::SLT,
			_ => unreachable!(),
		}
	} else {
		op
	};
	op = if take_reverse {
		match op {
			CompOp::SLT => CompOp::SGT,
			CompOp::SLE => CompOp::SGE,
			CompOp::SGT => CompOp::SLT,
			CompOp::SGE => CompOp::SLE,
			_ => unreachable!(),
		}
	} else {
		op
	};
	op
}
