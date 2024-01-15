pub fn align16(x: i32) -> i32 {
	(x + 15) & -16
}

pub fn is_pow2(x: i32) -> bool {
	x & (x - 1) == 0
}

pub fn increment(x: &mut i32) -> i32 {
	*x += 1;
	*x
}
