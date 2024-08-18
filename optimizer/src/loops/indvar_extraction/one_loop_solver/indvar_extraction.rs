use std::collections::HashSet;

use llvm::{compute_two_value, ArithInstr, ArithOp, LlvmTemp, Value};

use super::OneLoopSolver;

use utils::UseTemp;

impl<'a> OneLoopSolver<'a> {
	pub fn indvar_extraction(&mut self, phi_num: usize) {
		if let Some(info) = self.get_loop_info() {
			let phi_defs: Vec<LlvmTemp> = info
				.header
				.borrow()
				.phi_instrs
				.iter()
				.map(|phi| phi.target.clone())
				.collect();
			self.classify_usefulness(phi_num);
			let loop_cnt = self.compute_loop_cnt(&info);
			let mut headers_to_remove = HashSet::new();
			for phi_def in phi_defs.iter() {
				if !self.useful_variants.contains(phi_def) {
					if let Some(v) =
						self.extract_one_indvar(phi_def.clone(), loop_cnt.clone())
					{
						let new_header = ArithInstr {
							target: phi_def.clone(),
							op: ArithOp::Add,
							var_type: phi_def.var_type,
							lhs: v.clone(),
							rhs: Value::Int(0),
						};
						self
							.new_invariant_instr
							.insert(phi_def.clone(), Box::new(new_header));
						self.place_temp_into_cfg(phi_def);
						headers_to_remove.insert(phi_def.clone());
					}
				} else {
					#[cfg(feature = "debug")]
					eprintln!("not extract indvar: {}", phi_def);
				}
			}
			info
				.header
				.borrow_mut()
				.phi_instrs
				.retain(|phi| !headers_to_remove.contains(&phi.target));
		}
	}
	pub fn extract_one_indvar(
		&mut self,
		header: LlvmTemp,
		loop_cnt: Value,
	) -> Option<Value> {
		let indvar = self.indvars[&header].clone();
		// TODO: 只展开 scale 为 1 的
		// TODO: 乘法改成双字乘法
		// TODO： zfp 归纳变量的初始值可能大于 p,需要展开一次循环
		if indvar.scale == Value::Int(1) {
			#[cfg(feature = "debug")]
			eprintln!(
				"extracting indvar: {} {} with loop_cnt: {}",
				header, indvar, loop_cnt
			);
			let mut compute_two_value =
				|a: &Value, b: &Value, op: ArithOp| -> Value {
					let (output, instr) =
						compute_two_value(a.clone(), b.clone(), op, self.temp_mgr);
					if let Some(instr) = instr {
						self
							.new_invariant_instr
							.insert(instr.get_write().unwrap().clone(), instr);
					}
					output
				};
			// k 的阶乘
			let mut fract = Value::Int(1);
			let mut coef = loop_cnt.clone();
			let mut sum = indvar.base.clone();
			for (index, step) in indvar.step.iter().enumerate() {
				let tmp1 = compute_two_value(&coef, step, ArithOp::MulD);
				let tmp2 = compute_two_value(&tmp1, &fract, ArithOp::Div);
				sum = compute_two_value(&sum, &tmp2, ArithOp::Add);
				fract = compute_two_value(
					&fract,
					&Value::Int(index as i32 + 2),
					ArithOp::MulD,
				);
				let cnt_minus_one = compute_two_value(
					&loop_cnt,
					&Value::Int(index as i32 + 1),
					ArithOp::SubD,
				);
				coef = compute_two_value(&coef, &cnt_minus_one, ArithOp::MulD);
			}
			if let Some(zfp) = indvar.zfp.as_ref() {
				sum = compute_two_value(&sum, zfp, ArithOp::RemD);
			}
			Some(sum)
		} else {
			#[cfg(feature = "debug")]
			eprintln!("not extract indvar: {} {}", header, indvar);
			None
		}
	}
}
