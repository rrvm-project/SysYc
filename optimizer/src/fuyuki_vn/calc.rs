use llvm::{
	ArithOp, CompOp,
	Value::{self, *},
};

fn bin_calc<Foo, Bar>(
	x: &Value,
	y: &Value,
	on_int: Foo,
	on_float: Bar,
) -> Option<Value>
where
	Foo: Fn(i32, i32) -> i32,
	Bar: Fn(f32, f32) -> f32,
{
	match (x, y) {
		(Int(x), Int(y)) => Some(Int(on_int(*x, *y))),
		(Int(x), Float(y)) => Some(Float(on_float(*x as f32, *y))),
		(Float(x), Int(y)) => Some(Float(on_float(*x, *y as f32))),
		(Float(x), Float(y)) => Some(Float(on_float(*x, *y))),
		_ => None,
	}
}

fn bin_comp<Foo, Bar>(
	x: &Value,
	y: &Value,
	on_int: Foo,
	on_float: Bar,
) -> Option<Value>
where
	Foo: Fn(i32, i32) -> bool,
	Bar: Fn(f32, f32) -> bool,
{
	match (x, y) {
		(Int(x), Int(y)) => Some(Int(on_int(*x, *y) as i32)),
		(Int(x), Float(y)) => Some(Int(on_float(*x as f32, *y) as i32)),
		(Float(x), Int(y)) => Some(Int(on_float(*x, *y as f32) as i32)),
		(Float(x), Float(y)) => Some(Int(on_float(*x, *y) as i32)),
		_ => None,
	}
}

#[rustfmt::skip]
pub fn arith_binaryop(x: &Value, op: ArithOp, y: &Value) -> Option<Value> {
	match op {
		ArithOp::Add => bin_calc(x, y, |x, y| -> i32 {x.wrapping_add(y)}, |x, y| -> f32 {x + y}),
		ArithOp::Sub => bin_calc(x, y, |x, y| -> i32 {x.wrapping_sub(y)}, |x, y| -> f32 {x - y}),
		ArithOp::Mul => bin_calc(x, y, |x, y| -> i32 {x.wrapping_mul(y)}, |x, y| -> f32 {x * y}),
		ArithOp::Div => bin_calc(x, y, |x, y| -> i32 {x.wrapping_div(y)}, |x, y| -> f32 {x / y}),
		ArithOp::Rem => bin_calc(x, y, |x, y| -> i32 {x.wrapping_rem(y)}, |_, _| -> f32 {unreachable!()}),
		_ => None
	}
}

#[rustfmt::skip]
pub fn comp_binaryop(x: &Value, op: CompOp, y: &Value) -> Option<Value> {
	match op {
		CompOp::SLT => bin_comp(x, y, |x, y| -> bool {x < y}, |x, y| -> bool {x < y}),
		CompOp::SLE => bin_comp(x, y, |x, y| -> bool {x <= y}, |x, y| -> bool {x <= y}),
		CompOp::SGT => bin_comp(x, y, |x, y| -> bool {x > y}, |x, y| -> bool {x > y}),
		CompOp::SGE => bin_comp(x, y, |x, y| -> bool {x >= y}, |x, y| -> bool {x >= y}),
		CompOp::EQ => bin_comp(x, y, |x, y| -> bool {x == y}, |x, y| -> bool {x == y}),
		CompOp::NE => bin_comp(x, y, |x, y| -> bool {x != y}, |x, y| -> bool {x != y}),
		_ => None
	}
}
