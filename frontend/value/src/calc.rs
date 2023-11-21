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
	Foo: Fn(i32, i32) -> Result<i32>,
	Bar: Fn(f32, f32) -> Result<f32>,
{
	if x.get_type() == VarType::I32 && y.get_type() == VarType::I32 {
		Ok(Value::Int(on_int(x.to_int()?, y.to_int()?)?))
	} else {
		Ok(Value::Float(on_float(x.to_float()?, y.to_float()?)?))
	}
}

fn bin_comp<Foo, Bar>(
	x: &Value,
	y: &Value,
	on_int: Foo,
	on_float: Bar,
) -> Result<Value>
where
	Foo: Fn(i32, i32) -> Result<bool>,
	Bar: Fn(f32, f32) -> Result<bool>,
{
	if x.get_type() == VarType::I32 && y.get_type() == VarType::I32 {
		Ok(Value::Int(on_int(x.to_int()?, y.to_int()?)? as i32))
	} else {
		Ok(Value::Int(on_float(x.to_float()?, y.to_float()?)? as i32))
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
		BinaryOp::Add => bin_calc(x, y, |x, y| -> Result<i32> {Ok(x.wrapping_add(y))}, |x, y| -> Result<f32> {Ok(x + y)}),
		BinaryOp::Sub => bin_calc(x, y, |x, y| -> Result<i32> {Ok(x.wrapping_sub(y))}, |x, y| -> Result<f32> {Ok(x - y)}),
		BinaryOp::Mul => bin_calc(x, y, |x, y| -> Result<i32> {Ok(x.wrapping_mul(y))}, |x, y| -> Result<f32> {Ok(x * y)}),
		BinaryOp::Div => bin_calc(x, y, |x, y| -> Result<i32> {Ok(x.wrapping_div(y))}, |x, y| -> Result<f32> {Ok(x / y)}),
		BinaryOp::Mod => bin_calc(x, y, |x, y| -> Result<i32> {Ok(x.wrapping_rem(y))}, |_, _| -> Result<f32> {Err(TypeError("float number can not be used in mod".to_string()))}),
		BinaryOp::LT => bin_comp(x, y, |x, y| -> Result<bool> {Ok(x < y)}, |x, y| -> Result<bool> {Ok(x < y)}),
		BinaryOp::LE => bin_comp(x, y, |x, y| -> Result<bool> {Ok(x <= y)}, |x, y| -> Result<bool> {Ok(x <= y)}),
		BinaryOp::GT => bin_comp(x, y, |x, y| -> Result<bool> {Ok(x > y)}, |x, y| -> Result<bool> {Ok(x > y)}),
		BinaryOp::GE => bin_comp(x, y, |x, y| -> Result<bool> {Ok(x >= y)}, |x, y| -> Result<bool> {Ok(x >= y)}),
		BinaryOp::EQ => bin_comp(x, y, |x, y| -> Result<bool> {Ok(x == y)}, |x, y| -> Result<bool> {Ok(x == y)}),
		BinaryOp::NE => bin_comp(x, y, |x, y| -> Result<bool> {Ok(x != y)}, |x, y| -> Result<bool> {Ok(x != y)}),
    BinaryOp::Assign => unreachable!(),
	}
}

fn una_calc<Foo, Bar>(x: &Value, on_int: Foo, on_float: Bar) -> Result<Value>
where
	Foo: Fn(i32) -> Result<i32>,
	Bar: Fn(f32) -> Result<f32>,
{
	if x.get_type() == VarType::I32 {
		Ok(Value::Int(on_int(x.to_int()?)?))
	} else {
		Ok(Value::Float(on_float(x.to_float()?)?))
	}
}

#[rustfmt::skip]
pub fn exec_unaryop(op: UnaryOp, x: &Value) -> Result<Value> {
	match op {
	  UnaryOp::Plus => una_calc(x, |x|-> Result<i32> {Ok(x)} ,|x|-> Result<f32> {Ok(x)}),
	  UnaryOp::Neg => una_calc(x, |x|-> Result<i32> {Ok(-x)} ,|x|-> Result<f32> {Ok(-x)}),
	  UnaryOp::Not => una_calc(x, |x|-> Result<i32> {Ok(!x)} ,|_|-> Result<f32> {Err(TypeError("NOT operation is only for int".to_string()))}),
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn internal() {
		println!(
			"{:?}",
			exec_binaryop(
				&Value::IntPtr((
					vec![],
					(
						3,
						[(vec![3, 10, 1], 1), (vec![3, 10, 2], 2)]
							.iter()
							.cloned()
							.collect()
					)
				)),
				BinaryOp::IDX,
				&Value::Int(1)
			)
		);

		println!(
			"{:?}",
			exec_binaryop(
				&Value::IntPtr((
					vec![1],
					(
						3,
						[(vec![3, 10, 1], 1), (vec![3, 10, 2], 2)]
							.iter()
							.cloned()
							.collect()
					)
				)),
				BinaryOp::IDX,
				&Value::Int(1)
			)
		);

		println!(
			"{:?}",
			exec_binaryop(
				&Value::IntPtr((
					vec![1, 2],
					(
						3,
						[(vec![3, 10, 1], 1), (vec![3, 10, 2], 2)]
							.iter()
							.cloned()
							.collect()
					)
				)),
				BinaryOp::IDX,
				&Value::Int(1)
			)
		);

		println!(
			"{:?}",
			exec_binaryop(&Value::Float(9.8), BinaryOp::Add, &Value::Int(1))
		);

		assert_eq!(
			format!(
				"{:?}",
				exec_binaryop(
					&Value::IntPtr((
						vec![3, 10],
						(
							3,
							[(vec![3, 10, 1], 1), (vec![3, 10, 2], 2)]
								.iter()
								.cloned()
								.collect()
						)
					)),
					BinaryOp::IDX,
					&Value::Int(1)
				)
			),
			"Ok(Int(1))"
		)
	}
}
