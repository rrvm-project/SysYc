use llvm::{ArithInstr, ArithOp, Value, VarType};

use crate::indvar::IndVar;

use super::IndVarSolver;

impl<'a> IndVarSolver<'a> {
	pub fn compute_two_indvar(
		&mut self,
		v1: IndVar,
		v2: IndVar,
		op: ArithOp,
	) -> Option<IndVar> {
		let mut zfp = None;
		if v1.is_zfp.is_some() && v2.is_zfp.is_some() {
			if v1.is_zfp == v2.is_zfp {
				zfp.clone_from(&v1.is_zfp);
			}
		} else {
			zfp = v1.is_zfp.clone().or(v2.is_zfp.clone());
		}
		match op {
			ArithOp::Add | ArithOp::Sub => {
				let mut add_a_const = |v1: IndVar, v2: IndVar| -> Option<IndVar> {
					if v1.scale == Value::Int(1) && v1.step == Value::Int(0) {
						let const_value = v1.base;
						let new_base =
							self.compute_two_value(v2.base, const_value.clone(), op);
						let tmp1 = self.compute_two_value(
							Value::Int(1),
							v2.scale.clone(),
							ArithOp::Sub,
						);
						let tmp2 =
							self.compute_two_value(const_value.clone(), tmp1, ArithOp::Mul);
						let new_step = self.compute_two_value(v2.step, tmp2, op);
						Some(IndVar::new(new_base, v2.scale, new_step, zfp.clone()))
					} else {
						None
					}
				};
				add_a_const(v1.clone(), v2.clone())
					.or_else(|| add_a_const(v2.clone(), v1.clone()))
					.or_else(|| {
						if v1.scale == v2.scale {
							let new_base =
								self.compute_two_value(v1.base.clone(), v2.base.clone(), op);
							let new_step = self.compute_two_value(v1.step, v2.step, op);
							Some(IndVar::new(new_base, v1.scale, new_step, zfp.clone()))
						} else {
							None
						}
					})
			}
			ArithOp::Mul => {
				let mut mul_a_const = |v1: IndVar, v2: IndVar| -> Option<IndVar> {
					if v1.scale == Value::Int(1) && v1.step == Value::Int(0) {
						let const_value = v1.base;
						let new_base =
							self.compute_two_value(v2.base, const_value.clone(), op);
						let new_step =
							self.compute_two_value(v2.step, const_value.clone(), op);
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
	pub fn compute_two_value(
		&mut self,
		v1: Value,
		v2: Value,
		op: ArithOp,
	) -> Value {
		// 只考虑 int
		match v1.clone() {
			Value::Int(i1) => match v2.clone() {
				Value::Int(i2) => {
					let i = match op {
						ArithOp::Add => i1 + i2,
						ArithOp::Mul => i1 * i2,
						ArithOp::Sub => i1 - i2,
						ArithOp::Div => i1 / i2,
						_ => unreachable!(),
					};
					Value::Int(i)
				}
				Value::Temp(t2) => match (i1, op) {
					(0, ArithOp::Add | ArithOp::Sub) | (1, ArithOp::Mul | ArithOp::Div) => v2,
					(0, ArithOp::Mul) => Value::Int(0),
					_ => {
						let target = self.mgr.new_temp(VarType::I32, false);
						let instr = ArithInstr {
							target: target.clone(),
							op,
							var_type: VarType::I32,
							lhs: Value::Temp(t2),
							rhs: Value::Int(i1),
						};
						self.new_invariant_instr.insert(target.clone(), Box::new(instr));
						Value::Temp(target)
					}
				},
				Value::Float(_) => {
					unreachable!("add_two_value: v2 is float");
				}
			},
			Value::Temp(t1) => match v2 {
				Value::Int(i2) => match (i2, op) {
					(0, ArithOp::Add | ArithOp::Sub) | (1, ArithOp::Mul | ArithOp::Div) => v1,
					(0, ArithOp::Mul) => v2,
					_ => {
						let target = self.mgr.new_temp(VarType::I32, false);
						let instr = ArithInstr {
							target: target.clone(),
							op,
							var_type: VarType::I32,
							lhs: Value::Temp(t1),
							rhs: Value::Int(i2),
						};
						self.new_invariant_instr.insert(target.clone(), Box::new(instr));
						Value::Temp(target)
					}
				},
				Value::Temp(t2) => {
					let target = self.mgr.new_temp(VarType::I32, false);
					let instr = ArithInstr {
						target: target.clone(),
						op,
						var_type: VarType::I32,
						lhs: Value::Temp(t1),
						rhs: Value::Temp(t2),
					};
					self.new_invariant_instr.insert(target.clone(), Box::new(instr));
					Value::Temp(target)
				}
				Value::Float(_) => {
					unreachable!("add_two_value: v2 is float");
				}
			},
			Value::Float(_) => {
				unreachable!("add_two_value: v1 is float");
			}
		}
	}
}
