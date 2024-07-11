use llvm::LlvmTemp;

#[derive(Debug, Clone)]
pub struct Range {
	pub(crate) lower: Option<RangeItem>,
	pub(crate) upper: Option<RangeItem>,
}

#[derive(Debug, Clone)]
pub enum RangeItem {
	IntValue(i32),
	FloatValue(f32),
	IntFuture(LlvmTemp, i32, i32),
	FloatFuture(LlvmTemp, i32, f32),
}

impl Range {
	pub fn fromi32(i: i32) -> Self {
		Range {
			lower: Some(RangeItem::IntValue(i)),
			upper: Some(RangeItem::IntValue(i)),
		}
	}
	pub fn fromf32(f: f32) -> Self {
		Range {
			lower: Some(RangeItem::FloatValue(f)),
			upper: Some(RangeItem::FloatValue(f)),
		}
	}
}
