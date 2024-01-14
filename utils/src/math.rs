pub fn align16(x: i32) -> i32 {
	(x + 15) & -16
}

pub fn is_pow2(x: i32) -> bool {
	x & (x - 1) == 0
}
