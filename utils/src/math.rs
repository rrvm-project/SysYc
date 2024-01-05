pub fn align16(x: i32) -> i32 {
	(x + 15) & -16
}
