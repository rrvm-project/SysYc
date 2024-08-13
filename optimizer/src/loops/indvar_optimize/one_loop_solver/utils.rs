use llvm::{compute_two_value, ArithOp, Value};

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
			if i >= step1.len() {
				new_step.push(step2[i].clone());
			} else if i >= step2.len() {
				new_step.push(step1[i].clone());
			} else {
				let (v, instr) = compute_two_value(
					step1[i].clone(),
					step2[i].clone(),
					op,
					self.temp_mgr,
				);
				instr.map(|i| {
					self.new_invariant_instr.insert(i.target.clone(), Box::new(i))
				});
				new_step.push(v);
			}
		}
		new_step
	}
}
