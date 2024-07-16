use std::result;

use llvm::LlvmTemp;

#[derive(Debug, Clone, PartialEq)]
pub struct Range {
	pub(crate) lower: Option<RangeItem>,
	pub(crate) upper: Option<RangeItem>,
	pub(crate) contra: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RangeItem {
	IntValue(i32),
	FloatValue(f32),
	IntFuture(LlvmTemp, i32, i32),
	FloatFuture(LlvmTemp, i32, f32),
}

fn is_future(item: &RangeItem) -> bool {
	match item {
		RangeItem::IntValue(_) => false,
		RangeItem::FloatValue(_) => false,
		RangeItem::IntFuture(_, _, _) => true,
		RangeItem::FloatFuture(_, _, _) => true,
	}
}

fn max_update<T: PartialOrd + Copy>(this: &mut T, update: &T) -> bool {
	if *this < *update {
		*this = *update;
		true
	} else {
		false
	}
}

fn min_update<T: PartialOrd + Copy>(this: &mut T, update: &T) -> bool {
	if *this > *update {
		*this = *update;
		true
	} else {
		false
	}
}

impl RangeItem {
	pub fn max_update(&mut self, update: &Option<RangeItem>) -> bool {
		if let Some(update) = update {
			match (self, update) {
				(RangeItem::IntValue(this), RangeItem::IntValue(update)) => {
					max_update(this, update)
				}
				(RangeItem::FloatValue(this), RangeItem::FloatValue(update)) => {
					max_update(this, update)
				}
				(RangeItem::IntValue(_), RangeItem::FloatValue(_)) => {
					unreachable!("try to update int with float")
				}
				(RangeItem::FloatValue(_), RangeItem::IntValue(_)) => {
					unreachable!("try to update float with int")
				}
				_ => unreachable!("try to update or update with future"),
			}
		} else {
			false
		}
	}

	pub fn min_update(&mut self, update: &Option<RangeItem>) -> bool {
		if let Some(update) = update {
			match (self, update) {
				(RangeItem::IntValue(this), RangeItem::IntValue(update)) => {
					min_update(this, update)
				}
				(RangeItem::FloatValue(this), RangeItem::FloatValue(update)) => {
					min_update(this, update)
				}
				(RangeItem::IntValue(_), RangeItem::FloatValue(_)) => {
					unreachable!("try to update int with float")
				}
				(RangeItem::FloatValue(_), RangeItem::IntValue(_)) => {
					unreachable!("try to update float with int")
				}
				_ => unreachable!("try to update or update with future"),
			}
		} else {
			false
		}
	}
}

impl Range {
	pub fn fromi32(i: i32) -> Self {
		Range {
			lower: Some(RangeItem::IntValue(i)),
			upper: Some(RangeItem::IntValue(i)),
			contra: false,
		}
	}
	pub fn fromf32(f: f32) -> Self {
		Range {
			lower: Some(RangeItem::FloatValue(f)),
			upper: Some(RangeItem::FloatValue(f)),
			contra: false,
		}
	}
	pub fn inf() -> Self {
		Range {
			lower: None,
			upper: None,
			contra: false,
		}
	}
	pub fn contra() -> Self {
		Range {
			lower: None,
			upper: None,
			contra: true,
		}
	}

	fn contra_check(&mut self) {
		if self.contra {
			return;
		}
		self.contra = match (&self.lower, &self.upper) {
			(Some(l), Some(u)) => match (l, u) {
				(RangeItem::IntValue(l), RangeItem::IntValue(u)) => l > u,
				(RangeItem::FloatValue(l), RangeItem::FloatValue(u)) => l > u,
				_ => unreachable!(),
			},
			_ => false,
		};
	}

	// return true iff the range shirks
	pub fn intersection(&mut self, other: &Range) -> bool {
		let updated = if self.contra {
			false
		} else if other.contra {
			self.contra = true;
			true
		} else {
			let l = if let Some(this_lower) = &mut self.lower {
				this_lower.max_update(&other.lower)
			} else {
				// shrink the lower bound from -inf to other.lower
				self.lower = other.lower.clone();
				self.lower.is_some()
			};

			let u = if let Some(this_upper) = &mut self.upper {
				this_upper.min_update(&other.upper)
			} else {
				//shrink the upper bound from +inf to other.upper
				self.upper = other.upper.clone();
				self.upper.is_some()
			};
			l || u
		};
		self.contra_check();
		updated
	}

	//return true iff the range expands
	pub fn union(&mut self, other: &Range) -> bool {
		self.contra_check();
		if other.contra {
			false
		} else if self.contra {
			*self = other.clone();
			true
		} else {
			let l = if let Some(this_lower) = &mut self.lower {
				this_lower.min_update(&other.lower)
			} else {
				// -inf as the lower bound can never be expanded
				false
			};

			let u = if let Some(this_upper) = &mut self.upper {
				this_upper.max_update(&other.upper)
			} else {
				// + inf as the upper bound can  nevr be expanded
				false
			};
			l || u
		}
	}

	pub fn is_future(&self) -> bool {
		self.lower.iter().chain(self.upper.iter()).any(is_future)
	}
}
