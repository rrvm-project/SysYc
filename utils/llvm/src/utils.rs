use crate::{
	llvmop::Value,
	ArithInstr,
	ArithOp::{self},
	GEPInstr, LlvmInstr, LlvmTemp, LlvmTempManager, VarType,
};

pub fn unwrap_values(arr: Vec<&Value>) -> Vec<LlvmTemp> {
	arr.into_iter().flat_map(|v| v.unwrap_temp()).collect()
}

pub fn compute_two_value(
	v1: Value,
	v2: Value,
	op: ArithOp,
	temp_mgr: &mut LlvmTempManager,
) -> (Value, Option<LlvmInstr>) {
	use ArithOp::*;
	// 只考虑 int
	match (v1.clone(), v2.clone()) {
		(Value::Int(i1), Value::Int(i2)) => {
			let i = match op {
				Add => i1 + i2,
				AddD => i1 + i2,
				Mul => i1 * i2,
				MulD => i1 * i2,
				Sub => i1 - i2,
				SubD => i1 - i2,
				Div => i1 / i2,
				DivD => i1 / i2,
				Rem => i1 % i2,
				RemD => i1 % i2,
				_ => unreachable!(),
			};
			(Value::Int(i), None)
		}
		(Value::Float(f1), Value::Float(f2)) => {
			let f = match op {
				ArithOp::Fadd => f1 + f2,
				ArithOp::Fmul => f1 * f2,
				ArithOp::Fsub => f1 - f2,
				ArithOp::Fdiv => f1 / f2,
				_ => unreachable!(),
			};
			(Value::Float(f), None)
		}
		(Value::Int(i1), Value::Temp(t2)) => {
			assert!(t2.var_type != VarType::F32);
			match (i1, op) {
				(0, ArithOp::Add | ArithOp::AddD)
				| (1, ArithOp::Mul | ArithOp::MulD) => (v2, None),
				(0, Mul | MulD) => (Value::Int(0), None),
				_ => {
					assert!(
						t2.var_type != VarType::I32Ptr && t2.var_type != VarType::F32Ptr
					);
					let target = temp_mgr.new_temp(t2.var_type, false);
					let instr: LlvmInstr = Box::new(ArithInstr {
						target: target.clone(),
						op,
						var_type: t2.var_type,
						lhs: Value::Int(i1),
						rhs: Value::Temp(t2),
					});
					(Value::Temp(target), Some(instr))
				}
			}
		}
		(Value::Float(f1), Value::Temp(t2)) => {
			assert!(t2.var_type == VarType::F32);
			match (f1, op) {
				(0.0, ArithOp::Fadd) | (1.0, ArithOp::Fmul) => (v2, None),
				(0.0, ArithOp::Fmul) => (Value::Float(0.0), None),
				_ => {
					let target = temp_mgr.new_temp(t2.var_type, false);
					let instr = Box::new(ArithInstr {
						target: target.clone(),
						op,
						var_type: t2.var_type,
						lhs: Value::Float(f1),
						rhs: Value::Temp(t2),
					});
					(Value::Temp(target), Some(instr))
				}
			}
		}
		(Value::Temp(t1), Value::Int(i2)) => {
			assert!(t1.var_type != VarType::F32);
			match (i2, op) {
				(0, ArithOp::Add | ArithOp::AddD)
				| (1, ArithOp::Mul | ArithOp::MulD) => (v1, None),
				(0, ArithOp::Mul) => (Value::Int(0), None),
				_ => {
					let target = temp_mgr.new_temp(t1.var_type, false);
					let instr: LlvmInstr = if t1.var_type == VarType::I32Ptr
						|| t1.var_type == VarType::F32Ptr
					{
						Box::new(GEPInstr {
							target: target.clone(),
							var_type: t1.var_type,
							addr: Value::Temp(t1),
							offset: Value::Int(i2),
						})
					} else {
						Box::new(ArithInstr {
							target: target.clone(),
							op,
							var_type: t1.var_type,
							lhs: Value::Temp(t1),
							rhs: Value::Int(i2),
						})
					};
					(Value::Temp(target), Some(instr))
				}
			}
		}
		(Value::Temp(t1), Value::Float(f2)) => {
			assert!(t1.var_type == VarType::F32);
			match (f2, op) {
				(0.0, ArithOp::Fadd | ArithOp::Fsub)
				| (1.0, ArithOp::Fmul | ArithOp::Fdiv) => (v1, None),
				(0.0, ArithOp::Fmul) => (Value::Float(0.0), None),
				_ => {
					let target = temp_mgr.new_temp(t1.var_type, false);
					let instr = ArithInstr {
						target: target.clone(),
						op,
						var_type: t1.var_type,
						lhs: Value::Temp(t1),
						rhs: Value::Float(f2),
					};
					(Value::Temp(target), Some(Box::new(instr)))
				}
			}
		}
		(Value::Temp(t1), Value::Temp(t2)) => {
			assert!(
				t1.var_type == VarType::I32
					|| t2.var_type == VarType::I32
					|| t1.var_type == VarType::F32
					|| t2.var_type == VarType::F32
			);
			assert!(t2.var_type != VarType::I32Ptr && t2.var_type != VarType::F32Ptr);
			if t1.var_type == VarType::I32Ptr || t1.var_type == VarType::F32Ptr {
				let target = temp_mgr.new_temp(t1.var_type, false);
				let instr = Box::new(GEPInstr {
					target: target.clone(),
					var_type: t1.var_type,
					addr: Value::Temp(t1),
					offset: Value::Temp(t2),
				});
				(Value::Temp(target), Some(instr))
			} else {
				let target = temp_mgr.new_temp(t1.var_type, false);
				let instr = Box::new(ArithInstr {
					target: target.clone(),
					op,
					var_type: t1.var_type,
					lhs: Value::Temp(t1),
					rhs: Value::Temp(t2),
				});
				(Value::Temp(target), Some(instr))
			}
		}
		_ => {
			unreachable!();
		}
	}
}
