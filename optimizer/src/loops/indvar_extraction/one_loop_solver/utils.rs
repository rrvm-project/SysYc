use llvm::{compute_two_value, ArithOp, Value};

use crate::loops::indvar::IndVar;

use super::OneLoopSolver;

impl<'a> OneLoopSolver<'a> {
	pub fn compute_two_vec_values(
		&mut self,
		step1: &[Value],
		step2: &[Value],
		op: ArithOp,
	) -> Vec<Value> {
		let mut new_step = Vec::new();
		for i in 0..step1.len().max(step2.len()) {
			let v1 = if i < step1.len() {
				step1[i].clone()
			} else {
				Value::Int(0)
			};
			let v2 = if i < step2.len() {
				step2[i].clone()
			} else {
				Value::Int(0)
			};
			let (v, instr) = compute_two_value(v1, v2, op, self.temp_mgr);
			instr.map(|i| {
				self.new_invariant_instr.insert(i.get_write().unwrap().clone(), i)
			});
			new_step.push(v);
		}
		new_step
	}
	pub fn compute_two_indvar(
		&mut self,
		v1: IndVar,
		v2: IndVar,
		op: ArithOp,
	) -> Option<IndVar> {
		// 仅当 zfp 值相同并且再被 mod 了同一个 p 值时，才又是一个归纳变量
		if v1.zfp.is_some() || v2.zfp.is_some() {
			return None;
		}
		match op {
			ArithOp::Add | ArithOp::Sub => {
				if v1.scale == v2.scale {
					let (new_base, instr) = compute_two_value(
						v1.base.clone(),
						v2.base.clone(),
						op,
						self.temp_mgr,
					);
					instr.map(|i| {
						self.new_invariant_instr.insert(i.get_write().unwrap().clone(), i)
					});
					let new_step = self.compute_two_vec_values(&v1.step, &v2.step, op);
					Some(IndVar::new(new_base, v1.scale, new_step, None))
				} else {
					None
				}
			}
			ArithOp::Mul => {
				let mut mul_a_const = |v1: IndVar, v2: IndVar| -> Option<IndVar> {
					// 只乘常数
					if v1.scale == Value::Int(1) && v1.step.is_empty() {
						let const_value = v1.base;
						let (new_base, instr) = compute_two_value(
							v2.base,
							const_value.clone(),
							op,
							self.temp_mgr,
						);
						instr.map(|i| {
							self.new_invariant_instr.insert(i.get_write().unwrap().clone(), i)
						});
						let step2 = vec![const_value.clone(); v2.step.len()];
						let new_step = self.compute_two_vec_values(&v2.step, &step2, op);
						Some(IndVar::new(new_base, v2.scale, new_step, None))
					} else {
						None
					}
				};
				mul_a_const(v1.clone(), v2.clone())
					.or_else(|| mul_a_const(v2.clone(), v1.clone()))
			}
			_ => None,
		}
	}
}
