use utils::{errors::Result, SysycError::*};

use llvm::{
	llvmop::{ArithOp, CompOp},
	llvmvar::VarType,
	Value,
};

fn bin_calc<Foo, Bar>(
	x: &Value,
	y: &Value,
	on_int: Foo,
	on_float: Bar,
) -> Result<Value>
where
	Foo: Fn(i32, i32) -> i32,
	Bar: Fn(f32, f32) -> f32,
{
	if x.get_type() == VarType::I32 || y.get_type() == VarType::I32 {
		Ok(Value::Int(on_int(x.to_int()?, y.to_int()?)))
	} else {
		Ok(Value::Float(on_float(x.to_float()?, y.to_float()?)))
	}
}

fn bin_comp<Foo, Bar>(
	x: &Value,
	y: &Value,
	on_int: Foo,
	on_float: Bar,
) -> Result<Value>
where
	Foo: Fn(i32, i32) -> bool,
	Bar: Fn(f32, f32) -> bool,
{
	if x.get_type() == VarType::I32 || y.get_type() == VarType::I32 {
		Ok(Value::Int(on_int(x.to_int()?, y.to_int()?) as i32))
	} else {
		Ok(Value::Int(on_float(x.to_float()?, y.to_float()?) as i32))
	}
}

#[rustfmt::skip]
fn arith_binaryop_with_err(x: &Value, op: ArithOp, y: &Value) -> Result<Value> {
	match op {
		ArithOp::Add => bin_calc(x, y, |x, y| -> i32 {x.wrapping_add(y)}, |x, y| -> f32 {x + y}),
		ArithOp::Sub => bin_calc(x, y, |x, y| -> i32 {x.wrapping_sub(y)}, |x, y| -> f32 {x - y}),
		ArithOp::Mul => bin_calc(x, y, |x, y| -> i32 {x.wrapping_mul(y)}, |x, y| -> f32 {x * y}),
		ArithOp::Div => bin_calc(x, y, |x, y| -> i32 {x.wrapping_div(y)}, |x, y| -> f32 {x / y}),
		ArithOp::Rem => bin_calc(x, y, |x, y| -> i32 {x.wrapping_rem(y)}, |_, _| -> f32 {unreachable!()}),
		_ => Err(SystemError("".to_string()))
	}
}

pub fn arith_binaryop(x: &Value, op: ArithOp, y: &Value) -> Option<Value> {
	match arith_binaryop_with_err(x, op, y) {
		Ok(v) => Some(v),
		_ => None,
	}
}

#[rustfmt::skip]
fn comp_binaryop_with_err(x: &Value, op: CompOp, y: &Value) -> Result<Value> {
	match op {
		CompOp::SLT => bin_comp(x, y, |x, y| -> bool {x < y}, |x, y| -> bool {x < y}),
		CompOp::SLE => bin_comp(x, y, |x, y| -> bool {x <= y}, |x, y| -> bool {x <= y}),
		CompOp::SGT => bin_comp(x, y, |x, y| -> bool {x > y}, |x, y| -> bool {x > y}),
		CompOp::SGE => bin_comp(x, y, |x, y| -> bool {x >= y}, |x, y| -> bool {x >= y}),
		CompOp::EQ => bin_comp(x, y, |x, y| -> bool {x == y}, |x, y| -> bool {x == y}),
		CompOp::NE => bin_comp(x, y, |x, y| -> bool {x != y}, |x, y| -> bool {x != y}),
		_ => Err(SystemError("".to_string()))
	}
}

pub fn comp_binaryop(x: &Value, op: CompOp, y: &Value) -> Option<Value> {
	match comp_binaryop_with_err(x, op, y) {
		Ok(v) => Some(v),
		_ => None,
	}
}
