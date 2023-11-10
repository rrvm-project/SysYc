use ast::{BinaryOp, UnaryOp};
use ir_type::builtin_type::{BaseType, IRType};
use utils::{CompileConstValue, SysycError};

use std::collections::HashMap;

pub fn get_value() {}

fn get_type(x: &CompileConstValue) -> &str {
	match x {
		CompileConstValue::Int(_) => "int",
		CompileConstValue::Float(_) => "float",
		CompileConstValue::IntArray(_) => "int array",
		CompileConstValue::FloatArray(_) => "float array",
	}
}

fn get_type_mock(x: &IRType) -> Result<CompileConstValue, SysycError> {
	match (x.base_type, x.dims.len()) {
		(BaseType::Int, 0) => Ok(CompileConstValue::Int(1)),
		(BaseType::Int, _) => Ok(CompileConstValue::IntArray(HashMap::new())),
		(BaseType::Float, 0) => Ok(CompileConstValue::Float(1.0)),
		(BaseType::Float, _) => Ok(CompileConstValue::FloatArray(HashMap::new())),
		_ => Err(SysycError::SyntaxError(format!(
			"type {:?} is not available in expressions",
			x
		))),
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

fn error_unary_op(op: &str, rhs: &CompileConstValue) -> String {
	format!(
		"invalid operands of type and {} to  unary{}",
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
	logical: bool, // TODO 改成int_op 和 float_op返回一个enum以去掉logical!
) -> Result<CompileConstValue, SysycError> {
	if let (CompileConstValue::Int(x), CompileConstValue::Int(y)) = (lhs, rhs) {
		return Ok(CompileConstValue::Int(int_op(*x, *y)));
	}
	let err_msg = error_binary_op(lhs, op_name, rhs);
	let x =
		value_unwarp_f32(lhs).ok_or(SysycError::SyntaxError(err_msg.clone()))?;
	let y = value_unwarp_f32(rhs).ok_or(SysycError::SyntaxError(err_msg))?;
	if logical {
		if float_op(x, y) == 0.0 {
			Ok(CompileConstValue::Int(0))
		} else {
			Ok(CompileConstValue::Int(1))
		}
	} else {
		Ok(CompileConstValue::Float(float_op(x, y)))
	}
}

fn op_y(
	rhs: &CompileConstValue,
	op_name: &str,
	int_op: fn(i32) -> i32,
	float_op: fn(f32) -> f32,
	logical: bool,
) -> Result<CompileConstValue, SysycError> {
	if let CompileConstValue::Int(x) = rhs {
		Ok(CompileConstValue::Int(int_op(*x)))
	} else if let CompileConstValue::Float(x) = rhs {
		if logical {
			if float_op(*x) == 0.0 {
				Ok(CompileConstValue::Int(0))
			} else {
				Ok(CompileConstValue::Int(1))
			}
		} else {
			Ok(CompileConstValue::Float(float_op(*x)))
		}
	} else {
		Err(SysycError::SyntaxError(error_unary_op(op_name, rhs)))
	}
}

pub fn evaluate_binary(
	lhs: &CompileConstValue,
	op: &BinaryOp,
	rhs: &CompileConstValue,
) -> Result<CompileConstValue, SysycError> {
	// println!("{:?}, {:?}, {:?}", lhs, op ,rhs);
	match op {
		BinaryOp::Add => x_op_y(
			lhs,
			rhs,
			"add",
			|x, y| x.wrapping_add(y),
			|x, y| x + y,
			false,
		),
		BinaryOp::Assign => x_op_y(lhs, rhs, "assign", |_x, y| y, |_x, y| y, false),
		BinaryOp::Div => x_op_y(
			lhs,
			rhs,
			"div",
			|x, y| x.wrapping_div(y),
			|x, y| x / y,
			false,
		),
		BinaryOp::EQ => x_op_y(
			lhs,
			rhs,
			"eq",
			|x, y| if x == y { 1 } else { 0 },
			|x, y| if x == y { 1.0 } else { 0.0 },
			true,
		),
		BinaryOp::GE => x_op_y(
			lhs,
			rhs,
			"ge",
			|x, y| if x >= y { 1 } else { 0 },
			|x, y| if x >= y { 1.0 } else { 0.0 },
			true,
		),
		BinaryOp::GT => x_op_y(
			lhs,
			rhs,
			"gt",
			|x, y| if x > y { 1 } else { 0 },
			|x, y| if x > y { 1.0 } else { 0.0 },
			true,
		),
		BinaryOp::LE => x_op_y(
			lhs,
			rhs,
			"le",
			|x, y| if x <= y { 1 } else { 0 },
			|x, y| if x <= y { 1.0 } else { 0.0 },
			true,
		),
		BinaryOp::LT => x_op_y(
			lhs,
			rhs,
			"lt",
			|x, y| if x < y { 1 } else { 0 },
			|x, y| if x < y { 1.0 } else { 0.0 },
			true,
		),
		BinaryOp::Mod => {
			// TODO uncertain about correctness in float!
			x_op_y(lhs, rhs, "mod", |x, y| x % y, |x, y| x % y, false)
		}
		BinaryOp::Mul => {
			// TODO uncertain about correctness in float!
			x_op_y(
				lhs,
				rhs,
				"mul",
				|x, y| x.wrapping_mul(y),
				|x, y| x * y,
				false,
			)
		}
		BinaryOp::NE => x_op_y(
			lhs,
			rhs,
			"eq",
			|x, y| if x != y { 1 } else { 0 },
			|x, y| if x != y { 1.0 } else { 0.0 },
			true,
		),
		BinaryOp::Sub => x_op_y(
			lhs,
			rhs,
			"add",
			|x, y| x.wrapping_add(y.wrapping_neg()),
			|x, y| x - y,
			false,
		),
	}
}

pub fn evaluate_unary(
	op: &UnaryOp,
	rhs: &CompileConstValue,
) -> Result<CompileConstValue, SysycError> {
	match op {
		UnaryOp::Neg => op_y(rhs, "neg", |x| x.wrapping_neg(), |x| -x, false),
		UnaryOp::Plus => op_y(rhs, "plus", |x| x, |x| x, false),
		UnaryOp::Not => op_y(
			rhs,
			"not",
			|x| match x {
				0 => 1,
				_ => 0,
			},
			|x| {
				if x == 0.0 {
					0.0
				} else {
					1.0
				}
			},
			true,
		),
	}
}

pub fn type_unary(op: &UnaryOp, rhs: &IRType) -> Result<IRType, SysycError> {
	let mock_rhs = get_type_mock(rhs)?;
	let result = evaluate_unary(op, &mock_rhs)?;
	match result {
		CompileConstValue::Int(_) => Ok(IRType {
			base_type: BaseType::Int,
			dims: vec![],
			is_const: false,
		}),
		CompileConstValue::Float(_) => Ok(IRType {
			base_type: BaseType::Float,
			dims: vec![],
			is_const: false,
		}),
		_ => Err(SysycError::SyntaxError(
			"Result of unary operation must be scalar!".to_string(),
		)),
	}
}

pub fn type_binary(
	lhs: &IRType,
	op: &BinaryOp,
	rhs: &IRType,
) -> Result<IRType, SysycError> {
	let mock_rhs = get_type_mock(rhs)?;
	let mock_lhs = get_type_mock(lhs)?;
	let result = evaluate_binary(&mock_lhs, op, &mock_rhs)?;
	match result {
		CompileConstValue::Int(_) => Ok(IRType {
			base_type: BaseType::Int,
			dims: vec![],
			is_const: false,
		}),
		CompileConstValue::Float(_) => Ok(IRType {
			base_type: BaseType::Float,
			dims: vec![],
			is_const: false,
		}),
		_ => Err(SysycError::SyntaxError(
			"Result of binary operation must be scalar!".to_string(),
		)),
	}
}
