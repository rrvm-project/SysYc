use std::collections::HashMap;

use utils::{errors::Result, SysycError::*};

use crate::{BType, BinaryOp, UnaryOp, Value};

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
	if x.get_type() == BType::Int || y.get_type() == BType::Int {
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
	if x.get_type() == BType::Int || y.get_type() == BType::Int {
		Ok(Value::Int(on_int(x.to_int()?, y.to_int()?) as i32))
	} else {
		Ok(Value::Int(on_float(x.to_float()?, y.to_float()?) as i32))
	}
}

fn get_index<T: Into<Value> + Default>(
	index: &[usize],
	x: &HashMap<usize, T>,
	pos: usize,
) -> Result<Value>
where
	T: Into<Value> + Default + Copy,
	(Vec<usize>, HashMap<usize, T>): Into<Value>,
{
	let v = index
		.first()
		.ok_or(TypeError("Try to deref a non-pointer value".to_string()))?;
	let len: usize = index[1..].iter().product();
	if pos > *v {
		return Err(TypeError("Index out of bounds".to_string()));
	}
	if index.len() == 1 {
		//应该在外界检查，使得非const的值不参与前端常量计算。
		Ok(x.get(&pos).unwrap_or(&T::default()).to_owned().into())
	} else {
		let mut indexed = HashMap::new();
		for i in pos * len..(pos + 1) * len {
			if let Some(value) = x.get(&i) {
				indexed.insert(i - pos * len, *value);
			}
		}

		Ok((index[1..].to_vec(), indexed).into())
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
				Value::IntPtr((index, arr)) => get_index(index, arr, pos as usize),
				Value::FloatPtr((index, arr)) => get_index(index, arr, pos as usize),
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
		BinaryOp::LOr => bin_comp(x, y, |x, y| -> bool {x != 0 || y != 0}, |_, _| -> bool {unreachable!()}),
		BinaryOp::LAnd => bin_comp(x, y, |x, y| -> bool {x != 0 && y != 0}, |_, _| -> bool {unreachable!()}),
    BinaryOp::Assign => unreachable!(),
	}
}

fn una_calc<Foo, Bar>(x: &Value, on_int: Foo, on_float: Bar) -> Result<Value>
where
	Foo: Fn(i32) -> i32,
	Bar: Fn(f32) -> f32,
{
	if x.get_type() == BType::Int {
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
	  UnaryOp::BitNot => una_calc(x, |x|-> i32 {!x} ,|_|-> f32 {unreachable!()}),
	  UnaryOp::Not => una_calc(x, |x|-> i32 {(x == 0) as i32} ,|_|-> f32 {unreachable!()}),
	}
}
