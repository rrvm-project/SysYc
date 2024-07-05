pub enum Range {
	IntRange(IntRange),
	FloatRange(FloatRange),
}
#[allow(dead_code)]
pub struct IntRange {
	lower: Option<i32>,
	upper: Option<i32>,
}
#[allow(dead_code)]
pub struct FloatRange {
	lower: Option<f32>,
	upper: Option<f32>,
}
