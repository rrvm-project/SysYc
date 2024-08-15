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

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct Range {
	pub start: i32,
	pub end: i32,
}

impl Default for Range {
	fn default() -> Self {
		Self {
			start: i32::MAX,
			end: i32::MIN,
		}
	}
}

impl Range {
	pub fn new(start: i32, end: i32) -> Self {
		Self { start, end }
	}
	pub fn contains(&self, other: &Self) -> bool {
		self.start <= other.start && self.end >= other.end
	}
	pub fn extend(&mut self, other: &Self) {
		self.start = self.start.min(other.start);
		self.end = self.end.max(other.end);
	}
	pub fn shirink(&mut self, other: &Self) {
		self.start = self.start.max(other.start);
		self.end = self.end.min(other.end);
	}
}
