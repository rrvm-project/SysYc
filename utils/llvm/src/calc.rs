use crate::{
	ArithOp,
	Value::{self, *},
};

fn bin_int_calc<Foo>(x: &Value, y: &Value, func: Foo) -> Option<Value>
where
	Foo: Fn(i32, i32) -> i32,
{
	match (x, y) {
		(Int(x), Int(y)) => Some(Int(func(*x, *y))),
		_ => None,
	}
}

pub fn exec_binaryop(x: &Value, op: ArithOp, y: &Value) -> Option<Value> {
	match op {
		ArithOp::Add => bin_int_calc(x, y, |x, y| -> i32 { x.wrapping_add(y) }),
		ArithOp::Sub => bin_int_calc(x, y, |x, y| -> i32 { x.wrapping_sub(y) }),
		ArithOp::Mul => bin_int_calc(x, y, |x, y| -> i32 { x.wrapping_mul(y) }),
		ArithOp::Div => bin_int_calc(x, y, |x, y| -> i32 { x.wrapping_div(y) }),
		ArithOp::Rem => bin_int_calc(x, y, |x, y| -> i32 { x.wrapping_rem(y) }),
		_ => None,
	}
}
