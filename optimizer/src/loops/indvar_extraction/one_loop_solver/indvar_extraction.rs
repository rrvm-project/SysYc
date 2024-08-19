use std::collections::HashSet;

use llvm::{
	compute_two_value, ArithInstr, ArithOp, LlvmInstr, LlvmTemp, LlvmTempManager,
	Value,
};

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
		// TODO: 只展开 scale / divisor 为 1 的, 或 step 为 0 的
		// TODO：zfp 归纳变量的初始值可能大于 p,需要展开一次循环
		if indvar.scale == Value::Int(1) && indvar.divisor == Value::Int(1) {
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
		} else if indvar.step == vec![Value::Int(0)] {
			if indvar.scale == indvar.divisor {
				return Some(indvar.base.clone());
			}
			if let Some(scale_power) = is_power_of_two(&indvar.scale) {
				if let Some(divisor_power) = is_power_of_two(&indvar.divisor) {
					let (scale_power, instr) = compute_two_value(
						Value::Int(scale_power),
						loop_cnt.clone(),
						ArithOp::Mul,
						self.temp_mgr,
					);
					instr.and_then(|i| {
						self.new_invariant_instr.insert(i.get_write().unwrap().clone(), i)
					});
					let (divisor_power, instr) = compute_two_value(
						Value::Int(divisor_power),
						loop_cnt.clone(),
						ArithOp::Mul,
						self.temp_mgr,
					);
					instr.and_then(|i| {
						self.new_invariant_instr.insert(i.get_write().unwrap().clone(), i)
					});
					let (scale_power, instr) =
						compute_two_power(&scale_power, self.temp_mgr);
					instr.and_then(|i| {
						self.new_invariant_instr.insert(i.get_write().unwrap().clone(), i)
					});
					let (divisor_power, instr) =
						compute_two_power(&divisor_power, self.temp_mgr);
					instr.and_then(|i| {
						self.new_invariant_instr.insert(i.get_write().unwrap().clone(), i)
					});
					let (tmp3, instr) = compute_two_value(
						indvar.base,
						scale_power,
						ArithOp::MulD,
						self.temp_mgr,
					);
					instr.and_then(|i| {
						self.new_invariant_instr.insert(i.get_write().unwrap().clone(), i)
					});
					let (tmp4, instr) = compute_two_value(
						tmp3,
						divisor_power,
						ArithOp::DivD,
						self.temp_mgr,
					);
					instr.and_then(|i| {
						self.new_invariant_instr.insert(i.get_write().unwrap().clone(), i)
					});
					return Some(tmp4);
				}
			}
			#[cfg(feature = "debug")]
			eprintln!("not extract indvar(step==0): {} {}", header, indvar);
			None
		} else {
			#[cfg(feature = "debug")]
			eprintln!("not extract indvar: {} {}", header, indvar);
			None
		}
	}
}

// 判断一个 Value 是不是 2 的幂, 返回幂次
fn is_power_of_two(value: &Value) -> Option<i32> {
	match value {
		Value::Int(v) => {
			if v.count_ones() == 1 {
				Some(v.trailing_zeros() as i32)
			} else {
				None
			}
		}
		_ => None,
	}
}

// 计算 2 的 Value 次幂
fn compute_two_power(
	value: &Value,
	temp_mgr: &mut LlvmTempManager,
) -> (Value, Option<LlvmInstr>) {
	match value {
		Value::Int(v) => (Value::Int(1 << v), None),
		Value::Temp(t) => {
			let target = temp_mgr.new_temp(t.var_type, false);
			let instr = ArithInstr {
				target: target.clone(),
				var_type: target.var_type,
				op: ArithOp::ShlD,
				lhs: Value::Int(1),
				rhs: value.clone(),
			};
			(Value::Temp(target), Some(Box::new(instr)))
		}
		_ => unreachable!(),
	}
}
