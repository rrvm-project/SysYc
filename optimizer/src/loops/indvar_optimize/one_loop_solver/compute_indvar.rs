use llvm::{compute_two_value, ArithOp, Value};

use crate::loops::indvar::IndVar;

use super::OneLoopSolver;

impl<'a> OneLoopSolver<'a> {
	pub fn compute_two_indvar(
		&mut self,
		v1: IndVar,
		v2: IndVar,
		op: ArithOp,
	) -> Option<IndVar> {
		let zfp = match (v1.zfp.clone(), v2.zfp.clone()) {
			(Some(value1), Some(value2)) => {
				if value1 == value2 {
					v1.zfp.clone()
				} else {
					return None;
				}
			}
			_ => v1.zfp.clone().or(v2.zfp.clone()),
		};
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
						self.new_invariant_instr.insert(i.target.clone(), Box::new(i))
					});
					let new_step = self.compute_two_vec_values(&v1.step, &v2.step, op);
					Some(IndVar::new(new_base, v1.scale, new_step, zfp.clone()))
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
							self.new_invariant_instr.insert(i.target.clone(), Box::new(i))
						});
						let step2 = vec![const_value.clone(); v2.step.len()];
						let new_step = self.compute_two_vec_values(&v2.step, &step2, op);
						Some(IndVar::new(new_base, v2.scale, new_step, zfp.clone()))
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
