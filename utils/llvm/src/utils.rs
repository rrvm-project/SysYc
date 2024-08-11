use crate::{
	llvmop::Value, ArithInstr, ArithOp, LlvmTemp, LlvmTempManager, VarType,
};

pub fn unwrap_values(arr: Vec<&Value>) -> Vec<LlvmTemp> {
	arr.into_iter().flat_map(|v| v.unwrap_temp()).collect()
}

pub fn compute_two_value(
	v1: Value,
	v2: Value,
	op: ArithOp,
	temp_mgr: &mut LlvmTempManager,
) -> (Value, Option<ArithInstr>) {
	// 只考虑 int
	match (v1.clone(), v2.clone()) {
		(Value::Int(i1), Value::Int(i2)) => {
			let i = match op {
				ArithOp::Add => i1 + i2,
				ArithOp::Mul => i1 * i2,
				ArithOp::Sub => i1 - i2,
				ArithOp::Div => i1 / i2,
				_ => unreachable!(),
			};
			(Value::Int(i), None)
		}
		(Value::Int(i1), Value::Temp(t2)) => {
			assert!(t2.var_type != VarType::F32);
			match (i1, op) {
				(0, ArithOp::Add | ArithOp::Sub) | (1, ArithOp::Mul | ArithOp::Div) => {
					(v2, None)
				}
				(0, ArithOp::Mul) => (Value::Int(0), None),
				_ => {
					let target = temp_mgr.new_temp(t2.var_type, false);
					let instr = ArithInstr {
						target: target.clone(),
						op,
						var_type: t2.var_type,
						lhs: Value::Temp(t2),
						rhs: Value::Int(i1),
					};
					(Value::Temp(target), Some(instr))
				}
			}
		}
		(Value::Temp(t1), Value::Int(i2)) => {
			assert!(t1.var_type != VarType::F32);
			match (i2, op) {
				(0, ArithOp::Add | ArithOp::Sub) | (1, ArithOp::Mul | ArithOp::Div) => {
					(v2, None)
				}
				(0, ArithOp::Mul) => (Value::Int(0), None),
				_ => {
					let target = temp_mgr.new_temp(t1.var_type, false);
					let instr = ArithInstr {
						target: target.clone(),
						op,
						var_type: t1.var_type,
						lhs: Value::Temp(t1),
						rhs: Value::Int(i2),
					};
					(Value::Temp(target), Some(instr))
				}
			}
		}
		(Value::Temp(t1), Value::Temp(t2)) => {
			assert!(t1.var_type == VarType::I32 || t2.var_type == VarType::I32);
			let target_vartype = if t1.var_type == VarType::I32 {
				t2.var_type
			} else {
				t1.var_type
			};
			let target = temp_mgr.new_temp(target_vartype, false);
			let instr = ArithInstr {
				target: target.clone(),
				op,
				var_type: target_vartype,
				lhs: Value::Temp(t1),
				rhs: Value::Temp(t2),
			};
			(Value::Temp(target), Some(instr))
		}
		_ => {
			unreachable!();
		}
	}
}
