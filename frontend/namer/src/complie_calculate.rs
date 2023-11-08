use std::ops::AddAssign;

use ast::{BinaryOp, UnaryOp};
use utils::{CompileConstValue, SysycError};

pub fn get_value() {}

fn get_type(x: &CompileConstValue) -> &str {
	match x {
		CompileConstValue::Int(_) => "int",
		CompileConstValue::Float(_) => "float",
		CompileConstValue::IntArray(_) => "int array",
		CompileConstValue::FloatArray(_) => "float array",
		_ => todo!(),
	}
}

fn error_binary_op(
	lhs: &CompileConstValue,
	op: &str,
	rhs: &CompileConstValue,
) -> String {
	format!(
		"invalid operands of types {} and {} to binary {}",
		get_type(lhs),
		get_type(rhs),
		op
	)
}

fn value_unwarp_f32(x: &CompileConstValue) -> Option<f32> {
	match x {
		CompileConstValue::Int(v) => Some(*v as f32),
		CompileConstValue::Float(v) => Some(*v),
		_ => None,
	}
}

fn x_op_y(
	lhs: &CompileConstValue,
	rhs: &CompileConstValue,
	op_name: &str,
	int_op: fn(i32, i32) -> i32,
	float_op: fn(f32, f32) -> f32,
) -> Result<CompileConstValue, SysycError> {
	if let (CompileConstValue::Int(x), CompileConstValue::Int(y)) = (lhs, rhs) {
		return Ok(CompileConstValue::Int(int_op(*x, *y)));
	}
	let err_msg = error_binary_op(lhs, op_name, rhs);
	let x =
		value_unwarp_f32(&lhs).ok_or(SysycError::SyntaxError(err_msg.clone()))?;
	let y =
		value_unwarp_f32(&rhs).ok_or(SysycError::SyntaxError(err_msg.clone()))?;
	Ok(CompileConstValue::Float(float_op(x, y)))
}

pub fn evaluate_binary(
	lhs: &CompileConstValue,
	op: &BinaryOp,
	rhs: &CompileConstValue,
) -> Result<CompileConstValue, SysycError> {
	// println!("{:?}, {:?}, {:?}", lhs, op ,rhs);
	match op {
		BinaryOp::Add => {
			x_op_y(lhs, rhs, "add", |x, y| x.wrapping_add(y), |x, y| x + y)
		}

		_ => todo!(),
	}
}
