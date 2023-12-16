use llvm::llvmvar::VarType;
use utils::{errors::Result, SysycError::*};

use crate::{Array, BinaryOp, UnaryOp, Value};

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

fn get_index<T>(index: &[usize], x: &Array<T>, pos: i32) -> Value
where
	T: Into<Value> + Default + Copy,
	(Vec<usize>, Array<T>): Into<Value>,
{
	let (len, map) = x;
	let mut index = index.to_owned();
	index.push(pos as usize);
	if index.len() == *len {
		// UB may lead to any situation occurring, including receiving default values
		map.get(&index).copied().unwrap_or_default().into()
	} else {
		(index, (*len, map.clone())).into()
	}
}

#[rustfmt::skip]
pub fn exec_binaryop(x: &Value, op: BinaryOp, y: &Value) -> Result<Value> {
	match op {
		BinaryOp::IDX => {
			let pos = match y {
				Value::Int(v) => Ok(*v),
				_ => Err(TypeError("array can only be indexed by int".to_string())),
			}?;
			match x {
				Value::IntPtr((index, arr)) => Ok(get_index(index, arr, pos)),
				Value::FloatPtr((index, arr)) => Ok(get_index(index, arr, pos)),
				_ => Err(TypeError("only array can be indexed".to_string())),
			}
		}
		BinaryOp::Add => bin_calc(x, y, |x, y| -> i32 {x.wrapping_add(y)}, |x, y| -> f32 {x + y}),
		BinaryOp::Sub => bin_calc(x, y, |x, y| -> i32 {x.wrapping_sub(y)}, |x, y| -> f32 {x - y}),
		BinaryOp::Mul => bin_calc(x, y, |x, y| -> i32 {x.wrapping_mul(y)}, |x, y| -> f32 {x * y}),
		BinaryOp::Div => bin_calc(x, y, |x, y| -> i32 {x.wrapping_div(y)}, |x, y| -> f32 {x / y}),
		BinaryOp::Mod => bin_calc(x, y, |x, y| -> i32 {x.wrapping_rem(y)}, |_, _| -> f32 {unreachable!()}),
		BinaryOp::LT => bin_comp(x, y, |x, y| -> bool {x < y}, |x, y| -> bool {x < y}),
		BinaryOp::LE => bin_comp(x, y, |x, y| -> bool {x <= y}, |x, y| -> bool {x <= y}),
		BinaryOp::GT => bin_comp(x, y, |x, y| -> bool {x > y}, |x, y| -> bool {x > y}),
		BinaryOp::GE => bin_comp(x, y, |x, y| -> bool {x >= y}, |x, y| -> bool {x >= y}),
		BinaryOp::EQ => bin_comp(x, y, |x, y| -> bool {x == y}, |x, y| -> bool {x == y}),
		BinaryOp::NE => bin_comp(x, y, |x, y| -> bool {x != y}, |x, y| -> bool {x != y}),
    BinaryOp::Assign => unreachable!(),
	}
}

fn una_calc<Foo, Bar>(x: &Value, on_int: Foo, on_float: Bar) -> Result<Value>
where
	Foo: Fn(i32) -> i32,
	Bar: Fn(f32) -> f32,
{
	if x.get_type() == VarType::I32 {
		Ok(Value::Int(on_int(x.to_int()?)))
	} else {
		Ok(Value::Float(on_float(x.to_float()?)))
	}
}

#[rustfmt::skip]
pub fn exec_unaryop(op: UnaryOp, x: &Value) -> Result<Value> {
	match op {
	  UnaryOp::Plus => una_calc(x, |x|-> i32 {x} ,|x|-> f32 {x}),
	  UnaryOp::Neg => una_calc(x, |x|-> i32 {-x} ,|x|-> f32 {-x}),
	  UnaryOp::Not => una_calc(x, |x|-> i32 {!x} ,|_|-> f32 {unreachable!()}),
	}
}
