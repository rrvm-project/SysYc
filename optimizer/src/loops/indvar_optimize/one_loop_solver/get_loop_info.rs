use llvm::{compute_two_value, CompOp, LlvmInstrVariant, Value};

use crate::loops::loopinfo::LoopInfo;

use super::OneLoopSolver;

impl<'a: 'b, 'b> OneLoopSolver<'a, 'b> {
	// 如果不能确定循环总次数，则返回 None
	pub fn get_loop_info(&mut self) -> Option<LoopInfo> {
		let header = self.cur_loop.borrow().header.clone();
		let preheader = self.preheader.clone();
		let single_exit = match self.cur_loop.borrow().get_single_exit(
			&self
				.cur_loop
				.borrow()
				.blocks_without_subloops(&self.opter.func.cfg, &self.opter.loop_map),
			&self.opter.loop_map,
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
						self.opter.temp_graph.temp_to_instr[&branch_temp].instr.clone();
					match def_branch_temp.get_variant() {
						LlvmInstrVariant::CompInstr(inst) => {
							if matches!(
								inst.op,
								CompOp::SLT | CompOp::SLE | CompOp::SGT | CompOp::SGE
							) {
								let mut get_info = |cond_value: Value,
								                    end_value: Value,
								                    take_reverse: bool|
								 -> Option<LoopInfo> {
									if self.is_loop_invariant(&end_value) {
										if let Some(t) = cond_value.unwrap_temp() {
											if let Some(iv) = self.indvars.get(&t).cloned() {
												let new_op =
													convert_comp_op(inst.op, take_not, take_reverse);
												let loop_cnt = self.compute_loop_cnt(
													iv.base.clone(),
													iv.step.clone(),
													end_value.clone(),
													new_op,
												);
												#[cfg(feature = "debug")]
												eprintln!("get loop info: Found a loop to optimize with cond_temp: {} start: {}, step: {}, end: {}, op: {}, cnt: {}", t.clone(), iv.base.clone(), iv.step.clone(), end_value.clone(), new_op, loop_cnt.clone());
												let info = LoopInfo {
													indvars: self.indvars.clone(),
													branch_temp: branch_temp.clone(),
													comp_op: new_op,
													end: end_value,
													loop_cond_temp: t,
													loop_cnt,
													header: header.clone(),
													preheader: preheader.clone(),
													single_exit: single_exit.clone(),
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
	pub fn compute_loop_cnt(
		&mut self,
		start: Value,
		step: Value,
		end: Value,
		op: CompOp,
	) -> Value {
		match op {
			CompOp::SLT | CompOp::SGT => {
				// (end - start + step - 1) / step;
				let (tmp1, instr) = compute_two_value(
					end.clone(),
					start.clone(),
					llvm::ArithOp::Sub,
					self.opter.temp_mgr,
				);
				instr.map(|i| {
					self.new_invariant_instr.insert(i.target.clone(), Box::new(i))
				});
				let (tmp2, instr) = compute_two_value(
					tmp1,
					step.clone(),
					llvm::ArithOp::Add,
					self.opter.temp_mgr,
				);
				instr.map(|i| {
					self.new_invariant_instr.insert(i.target.clone(), Box::new(i))
				});
				let (tmp3, instr) = compute_two_value(
					tmp2,
					llvm::Value::Int(1),
					llvm::ArithOp::Sub,
					self.opter.temp_mgr,
				);
				instr.map(|i| {
					self.new_invariant_instr.insert(i.target.clone(), Box::new(i))
				});
				let (tmp4, instr) = compute_two_value(
					tmp3,
					step.clone(),
					llvm::ArithOp::Div,
					self.opter.temp_mgr,
				);
				instr.map(|i| {
					self.new_invariant_instr.insert(i.target.clone(), Box::new(i))
				});
				tmp4
			}
			CompOp::SLE | CompOp::SGE => {
				// (end - start + step) / step
				let (tmp1, instr) = compute_two_value(
					end.clone(),
					start.clone(),
					llvm::ArithOp::Sub,
					self.opter.temp_mgr,
				);
				instr.map(|i| {
					self.new_invariant_instr.insert(i.target.clone(), Box::new(i))
				});
				let (tmp2, instr) = compute_two_value(
					tmp1,
					step.clone(),
					llvm::ArithOp::Add,
					self.opter.temp_mgr,
				);
				instr.map(|i| {
					self.new_invariant_instr.insert(i.target.clone(), Box::new(i))
				});
				let (tmp3, instr) = compute_two_value(
					tmp2,
					step.clone(),
					llvm::ArithOp::Div,
					self.opter.temp_mgr,
				);
				instr.map(|i| {
					self.new_invariant_instr.insert(i.target.clone(), Box::new(i))
				});
				tmp3
			}
			_ => unreachable!(),
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
