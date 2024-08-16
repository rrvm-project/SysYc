use crate::{
	llvmop::Value, ArithInstr, ArithOp, GEPInstr, LlvmInstr, LlvmTemp,
	LlvmTempManager, VarType,
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
	// 只考虑 int, intPtr, floatPtr
	match (v1.clone(), v2.clone()) {
		(Value::Int(i1), Value::Int(i2)) => {
			let i = match op {
				ArithOp::Add => i1 + i2,
				ArithOp::Mul => i1 * i2,
				ArithOp::Sub => i1 - i2,
				ArithOp::Div => i1 / i2,
				ArithOp::Rem => i1 % i2,
				_ => unreachable!(),
			};
			(Value::Int(i), None)
		}
		(Value::Int(i1), Value::Temp(t2)) => {
			assert!(t2.var_type != VarType::F32);
			match (i1, op) {
				(0, ArithOp::Add) | (1, ArithOp::Mul) => (v2, None),
				(0, ArithOp::Mul) => (Value::Int(0), None),
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
		(Value::Temp(t1), Value::Int(i2)) => {
			assert!(t1.var_type != VarType::F32);
			match (i2, op) {
				(0, ArithOp::Add | ArithOp::Sub)
				| (1, ArithOp::Mul | ArithOp::Div | ArithOp::Rem) => (v1, None),
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
		(Value::Temp(t1), Value::Temp(t2)) => {
			assert!(t1.var_type == VarType::I32 || t2.var_type == VarType::I32);
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
