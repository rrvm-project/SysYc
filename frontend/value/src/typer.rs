use utils::errors::Result;

use crate::{
	calc::{exec_binaryop, exec_unaryop},
	BType, BinaryOp, UnaryOp, Value, VarType,
};
use std::collections::HashMap;

fn mock(tp: BType, dim_remain: usize) -> Result<Value> {
	match (tp, dim_remain) {
		(BType::Float, 0) => Ok(Value::Float(4.0)),
		(BType::Float, _) => {
			Ok(Value::FloatPtr((vec![1; dim_remain], (0, HashMap::new()))))
		}
		(BType::Int, 0) => Ok(Value::Int(4)),
		(BType::Int, _) => {
			Ok(Value::IntPtr((vec![1; dim_remain], (0, HashMap::new()))))
		}
	}
}

fn get_type(v: Value, l: &VarType, r: &VarType) -> Result<VarType> {
	match v {
		Value::Float(_) => Ok((l.0 & r.0, BType::Float, l.2.clone())),
		Value::Int(_) => Ok((l.0 & r.0, BType::Int, l.2.clone())),
		Value::IntPtr(_) => Ok((l.0 & r.0, BType::Int, l.2[1..].to_vec())),
		Value::FloatPtr(_) => Ok((l.0 & r.0, BType::Float, l.2[1..].to_vec())),
	}
}

pub fn type_for_unary(v: &VarType, op: UnaryOp) -> Result<VarType> {
	let value = mock(v.1, v.2.len())?;
	get_type(exec_unaryop(op, &value)?, v, v)
}

pub fn type_for_binary(
	l: &VarType,
	op: BinaryOp,
	r: &VarType,
) -> Result<VarType> {
	let value_l = mock(l.1, l.2.len())?;
	let value_r = mock(r.1, r.2.len())?;

	get_type(exec_binaryop(&value_l, op, &value_r)?, l, r)
}

// fn type_for_binary(tp1 : VarType, )

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn internal() {
		println!(
			"{:?}",
			type_for_unary(&(false, BType::Int, vec![]), UnaryOp::Not)
		);

		println!(
			"{:?}",
			type_for_binary(
				&(true, BType::Int, vec![]),
				BinaryOp::Add,
				&(true, BType::Float, vec![])
			)
		);

		println!(
			"{:?}",
			type_for_binary(
				&(true, BType::Int, vec![1, 2, 4]),
				BinaryOp::IDX,
				&(false, BType::Float, vec![])
			)
		);

		println!(
			"{:?}",
			type_for_binary(
				&(false, BType::Int, vec![1, 2, 4]),
				BinaryOp::IDX,
				&(false, BType::Int, vec![])
			)
		);

		// println!("{:?}",type_for_unary((false, BType::Int, vec![]), UnaryOp::Not));
		// println!("{:?}",type_for_unary(VarType::F32, 0, 0, UnaryOp::Not));
	}
}
