use llvm::LlvmTemp;
use utils::float_util::{f32_add_eps, f32_sub_eps};

#[derive(Clone, PartialEq)]
pub struct Range {
	pub lower: RangeItem,
	pub upper: RangeItem,
}

impl std::fmt::Debug for Range {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "[{:?},{:?}]", &self.lower, &self.upper)
	}
}

pub fn process_rem(
	min_a: i32,
	max_a: Option<i32>,
	min_b: i32,
	max_b: Option<i32>,
) -> Range {
	match (max_a, max_b) {
		(None, None) => Range::loweri32(0),
		(None, Some(max_b)) => {
			Range::from_items(RangeItem::IntValue(0), RangeItem::IntValue(max_b - 1))
		}
		(Some(max_a), None) => {
			if min_b > min_a {
				Range::from_items(
					RangeItem::IntValue(min_a),
					RangeItem::IntValue(max_a),
				)
			} else {
				Range::from_items(RangeItem::IntValue(0), RangeItem::IntValue(max_a))
			}
		}
		(Some(max_a), Some(max_b)) => {
			if max_b == min_b {
				let b = max_b;
				let len = max_a - min_a + 1;
				if len >= b {
					Range::from_items(RangeItem::IntValue(0), RangeItem::IntValue(b - 1))
				} else {
					let min_rem_a = min_a % b;
					let max_rem_a = max_a % b;
					if min_rem_a <= max_rem_a {
						Range::from_items(
							RangeItem::IntValue(min_rem_a),
							RangeItem::IntValue(max_rem_a),
						)
					} else {
						Range::from_items(
							RangeItem::IntValue(0),
							RangeItem::IntValue(b - 1),
						)
					}
				}
			} else if max_b > max_a {
				if min_b > max_a {
					Range::from_items(
						RangeItem::IntValue(min_a),
						RangeItem::IntValue(max_a),
					)
				} else {
					Range::from_items(RangeItem::IntValue(0), RangeItem::IntValue(max_a))
				}
			} else {
				Range::from_items(
					RangeItem::IntValue(0),
					RangeItem::IntValue(max_b - 1),
				)
			}
		}
	}
}

#[derive(Clone, PartialEq)]
pub enum RangeItem {
	IntValue(i32),
	FloatValue(f32),
	IntFuture(LlvmTemp, i32, i32),
	FloatFuture(LlvmTemp, i32, f32),
	PosInf,
	NegInf,
}

impl std::fmt::Debug for RangeItem {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::IntValue(arg0) => write!(f, "{:?}", arg0),
			Self::FloatValue(arg0) => write!(f, "{:?}", arg0),
			Self::IntFuture(arg0, arg1, arg2) => {
				write!(f, "{:?}@B{}+({})", arg0, arg1, arg2)
			}
			Self::FloatFuture(arg0, arg1, arg2) => {
				write!(f, "{:?}@B{}+({})", arg0, arg1, arg2)
			}
			Self::PosInf => write!(f, "+Inf"),
			Self::NegInf => write!(f, "-Inf"),
		}
	}
}
#[allow(dead_code)]
fn is_future(item: &RangeItem) -> bool {
	matches!(
		item,
		RangeItem::IntFuture(_, _, _) | RangeItem::FloatFuture(_, _, _)
	)
}

use utils::{errors::Result, SysycError::SystemError};

impl RangeItem {
	pub fn add(&self, other: &RangeItem) -> Result<RangeItem> {
		match (self, other) {
			(RangeItem::FloatFuture(_, _, _), _)
			| (RangeItem::IntFuture(_, _, _), _) => {
				unreachable!("unexpected future")
			}
			(_, RangeItem::FloatFuture(_, _, _))
			| (_, RangeItem::IntFuture(_, _, _)) => {
				unreachable!("unexpected future")
			}
			(RangeItem::NegInf, RangeItem::PosInf)
			| (RangeItem::PosInf, RangeItem::NegInf) => {
				Err(utils::SysycError::SystemError("Nan".to_string()))
			}

			(RangeItem::NegInf, _) | (_, RangeItem::NegInf) => Ok(RangeItem::NegInf),
			(RangeItem::PosInf, _) | (_, RangeItem::PosInf) => Ok(RangeItem::PosInf),
			(RangeItem::IntValue(this), RangeItem::IntValue(other)) => this
				.checked_add(*other)
				.map(RangeItem::IntValue)
				.ok_or(SystemError("int overflow".to_string())),
			(RangeItem::FloatValue(this), RangeItem::FloatValue(other)) => {
				Ok(RangeItem::FloatValue(*this + *other))
			}
			(RangeItem::IntValue(_), RangeItem::FloatValue(_))
			| (RangeItem::FloatValue(_), RangeItem::IntValue(_)) => unreachable!(),
		}
	}

	pub fn sub(&self, other: &RangeItem) -> Result<RangeItem> {
		match (self, other) {
			(RangeItem::FloatFuture(_, _, _), _)
			| (RangeItem::IntFuture(_, _, _), _) => {
				unreachable!("unexpected future")
			}
			(_, RangeItem::FloatFuture(_, _, _))
			| (_, RangeItem::IntFuture(_, _, _)) => {
				unreachable!("unexpected future")
			}
			(RangeItem::NegInf, RangeItem::NegInf)
			| (RangeItem::PosInf, RangeItem::PosInf) => {
				Err(utils::SysycError::SystemError("Nan".to_string()))
			}

			(RangeItem::NegInf, _) | (_, RangeItem::PosInf) => Ok(RangeItem::NegInf),
			(RangeItem::PosInf, _) | (_, RangeItem::NegInf) => Ok(RangeItem::PosInf),
			(RangeItem::IntValue(this), RangeItem::IntValue(other)) => this
				.checked_sub(*other)
				.map(RangeItem::IntValue)
				.ok_or(SystemError("int overflow".to_string())),
			(RangeItem::FloatValue(this), RangeItem::FloatValue(other)) => {
				Ok(RangeItem::FloatValue(*this - *other))
			}
			(RangeItem::IntValue(_), RangeItem::FloatValue(_))
			| (RangeItem::FloatValue(_), RangeItem::IntValue(_)) => unreachable!(),
		}
	}

	pub fn mul(&self, other: &RangeItem) -> Result<RangeItem> {
		match (self, other) {
			(RangeItem::FloatFuture(_, _, _), _)
			| (RangeItem::IntFuture(_, _, _), _) => {
				unreachable!("unexpected future")
			}
			(_, RangeItem::FloatFuture(_, _, _))
			| (_, RangeItem::IntFuture(_, _, _)) => {
				unreachable!("unexpected future")
			}
			(RangeItem::NegInf, RangeItem::NegInf)
			| (RangeItem::PosInf, RangeItem::PosInf) => Ok(RangeItem::PosInf),
			(RangeItem::NegInf, RangeItem::PosInf)
			| (RangeItem::PosInf, RangeItem::NegInf) => Ok(RangeItem::NegInf),

			(RangeItem::NegInf, RangeItem::IntValue(v))
			| (RangeItem::IntValue(v), RangeItem::NegInf) => Ok(match v {
				0 => RangeItem::IntValue(0),
				1.. => RangeItem::NegInf,
				..=-1 => RangeItem::PosInf,
			}),

			(RangeItem::NegInf, RangeItem::FloatValue(v))
			| (RangeItem::FloatValue(v), RangeItem::NegInf) => match v {
				_ if *v == 0.0 => Ok(RangeItem::FloatValue(0f32)),
				_ if *v > 0.0 => Ok(RangeItem::NegInf),
				_ if *v < 0.0 => Ok(RangeItem::PosInf),
				_ => Err(SystemError("unexpected float".to_string())),
			},

			(RangeItem::PosInf, RangeItem::IntValue(v))
			| (RangeItem::IntValue(v), RangeItem::PosInf) => Ok(match v {
				0 => RangeItem::IntValue(0),
				1.. => RangeItem::PosInf,
				..=-1 => RangeItem::NegInf,
			}),

			(RangeItem::PosInf, RangeItem::FloatValue(v))
			| (RangeItem::FloatValue(v), RangeItem::PosInf) => match v {
				_ if *v == 0.0 => Ok(RangeItem::FloatValue(0f32)),
				_ if *v > 0.0 => Ok(RangeItem::PosInf),
				_ if *v < 0.0 => Ok(RangeItem::NegInf),
				_ => Err(SystemError("unexpected float".to_string())),
			},

			(RangeItem::IntValue(this), RangeItem::IntValue(other)) => this
				.checked_mul(*other)
				.map(RangeItem::IntValue)
				.ok_or(SystemError("int overflow".to_string())),
			(RangeItem::FloatValue(this), RangeItem::FloatValue(other)) => {
				Ok(RangeItem::FloatValue(*this * *other))
			}
			(RangeItem::IntValue(_), RangeItem::FloatValue(_))
			| (RangeItem::FloatValue(_), RangeItem::IntValue(_)) => {
				// Ok(RangeItem::FloatValue(*i as f32 * *f))
				Err(SystemError("unexpected type".to_string()))
			}
		}
	}

	pub fn div(&self, other: &RangeItem) -> Result<RangeItem> {
		match (self, other) {
			(RangeItem::FloatFuture(_, _, _), _)
			| (RangeItem::IntFuture(_, _, _), _) => unreachable!("unexpected future"),
			(_, RangeItem::FloatFuture(_, _, _))
			| (_, RangeItem::IntFuture(_, _, _)) => unreachable!("unexpected future"),

			(RangeItem::NegInf, RangeItem::IntValue(v)) if *v > 0 => {
				Ok(RangeItem::NegInf)
			}
			(RangeItem::NegInf, RangeItem::FloatValue(v)) if *v > 0f32 => {
				Ok(RangeItem::NegInf)
			}
			(RangeItem::PosInf, RangeItem::IntValue(v)) if *v > 0 => {
				Ok(RangeItem::PosInf)
			}
			(RangeItem::PosInf, RangeItem::FloatValue(v)) if *v > 0f32 => {
				Ok(RangeItem::PosInf)
			}
			(RangeItem::NegInf, RangeItem::IntValue(v)) if *v < 0 => {
				Ok(RangeItem::PosInf)
			}
			(RangeItem::NegInf, RangeItem::FloatValue(v)) if *v < 0f32 => {
				Ok(RangeItem::PosInf)
			}
			(RangeItem::PosInf, RangeItem::IntValue(v)) if *v < 0 => {
				Ok(RangeItem::NegInf)
			}
			(RangeItem::PosInf, RangeItem::FloatValue(v)) if *v < 0f32 => {
				Ok(RangeItem::NegInf)
			}
			(RangeItem::IntValue(_), RangeItem::PosInf) => Ok(RangeItem::IntValue(0)),
			(RangeItem::IntValue(_), RangeItem::NegInf) => Ok(RangeItem::IntValue(0)),
			(RangeItem::FloatValue(_), RangeItem::PosInf) => {
				Ok(RangeItem::FloatValue(0f32))
			}
			(RangeItem::FloatValue(_), RangeItem::NegInf) => {
				Ok(RangeItem::FloatValue(0f32))
			}
			(RangeItem::FloatValue(a), RangeItem::FloatValue(b)) => {
				Ok(RangeItem::FloatValue(a / b))
			}
			(RangeItem::IntValue(a), RangeItem::IntValue(b)) => {
				Ok(RangeItem::IntValue(a / b))
			}

			_ => Err(SystemError("undefined".to_string())),
		}
	}

	pub fn max(&self, update: &RangeItem) -> RangeItem {
		match (self, update) {
			(RangeItem::FloatFuture(_, _, _), _)
			| (RangeItem::IntFuture(_, _, _), _) => {
				unreachable!("unexpected future")
			}
			(_, RangeItem::FloatFuture(_, _, _))
			| (_, RangeItem::IntFuture(_, _, _)) => {
				unreachable!("unexpected future")
			}
			(RangeItem::NegInf, _) | (_, RangeItem::PosInf) => update.clone(),
			(RangeItem::PosInf, _) | (_, RangeItem::NegInf) => self.clone(),
			(RangeItem::IntValue(this), RangeItem::IntValue(update)) => {
				RangeItem::IntValue(*this.max(update))
			}
			(RangeItem::FloatValue(this), RangeItem::FloatValue(update)) => {
				RangeItem::FloatValue(this.max(*update))
			}
			(RangeItem::IntValue(a), RangeItem::FloatValue(b)) => {
				if (*a as f32) < { *b } {
					update.clone()
				} else {
					self.clone()
				}
			}
			(RangeItem::FloatValue(a), RangeItem::IntValue(b)) => {
				if { *a } < (*b as f32) {
					update.clone()
				} else {
					self.clone()
				}
			}
		}
	}

	pub fn min(&self, update: &RangeItem) -> RangeItem {
		match (self, update) {
			(RangeItem::FloatFuture(_, _, _), _)
			| (RangeItem::IntFuture(_, _, _), _) => {
				unreachable!("unexpected future")
			}
			(_, RangeItem::FloatFuture(_, _, _))
			| (_, RangeItem::IntFuture(_, _, _)) => {
				unreachable!("unexpected future")
			}
			(RangeItem::NegInf, _) | (_, RangeItem::PosInf) => self.clone(),
			(RangeItem::PosInf, _) | (_, RangeItem::NegInf) => update.clone(),
			(RangeItem::IntValue(this), RangeItem::IntValue(update)) => {
				RangeItem::IntValue(*this.min(update))
			}
			(RangeItem::FloatValue(this), RangeItem::FloatValue(update)) => {
				RangeItem::FloatValue(this.min(*update))
			}
			_ => unreachable!("unexcepted type"),
		}
	}
}

impl PartialOrd for RangeItem {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		match (self, other) {
			(RangeItem::IntFuture(_, _, _), _) => None,
			(RangeItem::FloatFuture(_, _, _), _) => None,
			(_, RangeItem::IntFuture(_, _, _)) => None,
			(_, RangeItem::FloatFuture(_, _, _)) => None,

			(RangeItem::IntValue(a), RangeItem::IntValue(b)) => Some(a.cmp(b)),
			(RangeItem::IntValue(a), RangeItem::FloatValue(b)) => {
				let a = *a as f32;
				if a > *b {
					Some(std::cmp::Ordering::Greater)
				} else if a < *b {
					Some(std::cmp::Ordering::Less)
				} else if a == *b {
					Some(std::cmp::Ordering::Equal)
				} else {
					None
				}
			}
			(RangeItem::IntValue(_), RangeItem::PosInf) => {
				Some(std::cmp::Ordering::Less)
			}
			(RangeItem::IntValue(_), RangeItem::NegInf) => {
				Some(std::cmp::Ordering::Greater)
			}
			(RangeItem::FloatValue(a), RangeItem::IntValue(b)) => {
				let b = *b as f32;
				if *a > b {
					Some(std::cmp::Ordering::Greater)
				} else if *a < b {
					Some(std::cmp::Ordering::Less)
				} else if *a == b {
					Some(std::cmp::Ordering::Equal)
				} else {
					None
				}
			}
			(RangeItem::FloatValue(a), RangeItem::FloatValue(b)) => {
				if a > b {
					Some(std::cmp::Ordering::Greater)
				} else if a < b {
					Some(std::cmp::Ordering::Less)
				} else if a == b {
					Some(std::cmp::Ordering::Equal)
				} else {
					None
				}
			}
			(RangeItem::FloatValue(_), RangeItem::PosInf) => {
				Some(std::cmp::Ordering::Less)
			}
			(RangeItem::FloatValue(_), RangeItem::NegInf) => {
				Some(std::cmp::Ordering::Greater)
			}
			(RangeItem::PosInf, RangeItem::IntValue(_)) => {
				Some(std::cmp::Ordering::Greater)
			}
			(RangeItem::PosInf, RangeItem::FloatValue(_)) => {
				Some(std::cmp::Ordering::Greater)
			}
			(RangeItem::PosInf, RangeItem::PosInf) => Some(std::cmp::Ordering::Equal),
			(RangeItem::PosInf, RangeItem::NegInf) => {
				Some(std::cmp::Ordering::Greater)
			}
			(RangeItem::NegInf, RangeItem::IntValue(_)) => {
				Some(std::cmp::Ordering::Less)
			}
			(RangeItem::NegInf, RangeItem::FloatValue(_)) => {
				Some(std::cmp::Ordering::Less)
			}
			(RangeItem::NegInf, RangeItem::PosInf) => Some(std::cmp::Ordering::Less),
			(RangeItem::NegInf, RangeItem::NegInf) => Some(std::cmp::Ordering::Equal),
		}
	}
}

impl Range {
	pub fn fromi32(i: i32) -> Self {
		Range {
			lower: RangeItem::IntValue(i),
			upper: RangeItem::IntValue(i),
		}
	}
	pub fn fromf32(f: f32) -> Self {
		Range {
			lower: RangeItem::FloatValue(f),
			upper: RangeItem::FloatValue(f),
		}
	}

	pub fn loweri32(i: i32) -> Self {
		Range {
			lower: RangeItem::IntValue(i),
			upper: RangeItem::PosInf,
		}
	}
	#[allow(dead_code)]
	pub fn lowerf32(f: f32) -> Self {
		Range {
			lower: RangeItem::FloatValue(f),
			upper: RangeItem::PosInf,
		}
	}
	#[allow(dead_code)]
	pub fn upperi32(i: i32) -> Self {
		Range {
			lower: RangeItem::NegInf,
			upper: RangeItem::IntValue(i),
		}
	}
	#[allow(dead_code)]
	pub fn upperf32(f: f32) -> Self {
		Range {
			lower: RangeItem::NegInf,
			upper: RangeItem::FloatValue(f),
		}
	}

	pub fn from_items(lower: RangeItem, upper: RangeItem) -> Self {
		let mut result = Range { lower, upper };
		result.contra_check();
		result
	}

	pub fn inf() -> Self {
		Range {
			lower: RangeItem::NegInf,
			upper: RangeItem::PosInf,
		}
	}
	pub fn contra() -> Self {
		Range {
			lower: RangeItem::PosInf,
			upper: RangeItem::NegInf,
		}
	}

	pub fn contra_check(&mut self) -> bool {
		if match (&self.lower, &self.upper) {
			(RangeItem::IntValue(l), RangeItem::IntValue(u)) => l > u,
			(RangeItem::FloatValue(l), RangeItem::FloatValue(u)) => l > u,
			(RangeItem::PosInf, _) | (_, RangeItem::NegInf) => true,
			(_, RangeItem::PosInf) | (RangeItem::NegInf, _) => false,
			_ => false,
		} {
			self.lower = RangeItem::PosInf;
			self.upper = RangeItem::NegInf;
			true
		} else {
			false
		}
	}

	fn is_contra(&self) -> bool {
		match (&self.lower, &self.upper) {
			(RangeItem::IntValue(l), RangeItem::IntValue(u)) => l > u,
			(RangeItem::FloatValue(l), RangeItem::FloatValue(u)) => l > u,
			(RangeItem::PosInf, _) | (_, RangeItem::NegInf) => true,
			(_, RangeItem::PosInf) | (RangeItem::NegInf, _) => false,
			_ => unreachable!(),
		}
	}

	pub fn intersection(&self, other: &Range) -> Range {
		let mut result = Range {
			lower: self.lower.max(&other.lower),
			upper: self.upper.min(&other.upper),
		};
		result.contra_check();
		result
	}

	pub fn union(&self, other: &Range) -> Range {
		Range {
			lower: self.lower.min(&other.lower),
			upper: self.upper.max(&other.upper),
		}
	}
	#[allow(dead_code)]
	pub fn is_future(&self) -> bool {
		is_future(&self.lower) | is_future(&self.upper)
	}

	pub fn add(&self, other: &Range) -> Range {
		Range {
			lower: self.lower.add(&other.lower).ok().unwrap_or(RangeItem::NegInf),
			upper: self.upper.add(&other.upper).ok().unwrap_or(RangeItem::PosInf),
		}
	}

	pub fn sub(&self, other: &Range) -> Range {
		Range {
			lower: self.lower.sub(&other.upper).ok().unwrap_or(RangeItem::NegInf),
			upper: self.upper.sub(&other.lower).ok().unwrap_or(RangeItem::PosInf),
		}
	}

	pub fn mul(&self, other: &Range) -> Range {
		let mut results = vec![];
		if let Ok(u) = self.lower.mul(&other.lower) {
			results.push(u)
		}
		if let Ok(u) = self.lower.mul(&other.upper) {
			results.push(u)
		}
		if let Ok(u) = self.upper.mul(&other.lower) {
			results.push(u)
		}
		if let Ok(u) = self.upper.mul(&other.upper) {
			results.push(u)
		}
		if results.len() == 4 {
			Range {
				lower: results.iter().fold(RangeItem::PosInf, |x, y| x.min(y)),
				upper: results.iter().fold(RangeItem::NegInf, |x, y| x.max(y)),
			}
		} else {
			Range::inf()
		}
	}

	// Some(true) -> positive
	// Some(false) -> negative
	// None -> can not be determined
	pub fn positive(&self) -> Option<bool> {
		match (&self.lower, &self.upper) {
			(RangeItem::NegInf, RangeItem::IntValue(v)) if *v < 0 => Some(false),
			(RangeItem::NegInf, RangeItem::FloatValue(v)) if *v < 0f32 => Some(false),
			(RangeItem::IntValue(a), RangeItem::IntValue(b))
				if *a <= *b && *b < 0 =>
			{
				Some(false)
			}
			(RangeItem::FloatValue(a), RangeItem::FloatValue(b))
				if *a <= *b && *b < 0.0 =>
			{
				Some(false)
			}
			(RangeItem::IntValue(v), RangeItem::PosInf) if *v > 0 => Some(true),
			(RangeItem::FloatValue(v), RangeItem::PosInf) if *v > 0f32 => Some(true),
			(RangeItem::IntValue(a), RangeItem::IntValue(b))
				if *a <= *b && *a > 0 =>
			{
				Some(true)
			}
			(RangeItem::FloatValue(a), RangeItem::FloatValue(b))
				if *a <= *b && *a > 0.0 =>
			{
				Some(true)
			}
			_ => None,
		}
	}

	#[allow(dead_code)]
	pub fn is_int(&self) -> bool {
		matches!(
			(&self.lower, &self.upper),
			(RangeItem::IntValue(_), _) | (_, RangeItem::IntValue(_))
		)
	}

	#[allow(dead_code)]
	pub fn is_float(&self) -> bool {
		matches!(
			(&self.lower, &self.upper),
			(RangeItem::FloatValue(_), _) | (_, RangeItem::FloatValue(_))
		)
	}

	pub fn to_int(&self) -> Self {
		fn lower_trunc(v: f32) -> RangeItem {
			if v as f64 >= (std::i32::MIN + 1) as f64 {
				let i: i32 = v.floor() as i32;
				RangeItem::IntValue(i)
			} else {
				RangeItem::NegInf
			}
		}

		fn upper_trunc(v: f32) -> RangeItem {
			if v as f64 <= (std::i32::MAX - 1) as f64 {
				let i: i32 = v.ceil() as i32;
				RangeItem::IntValue(i)
			} else {
				RangeItem::PosInf
			}
		}

		let result = match (&self.lower, &self.upper) {
			(RangeItem::FloatValue(v), upper) => Range {
				lower: lower_trunc(*v),
				upper: upper.clone(),
			},
			(lower, upper) => Range {
				lower: lower.clone(),
				upper: upper.clone(),
			},
		};

		match (result.lower, result.upper) {
			(lower, RangeItem::FloatValue(v)) => Range {
				lower,
				upper: upper_trunc(v),
			},
			(lower, upper) => Range { lower, upper },
		}
	}

	pub fn to_float(&self) -> Self {
		fn lower_trunc(v: i32) -> RangeItem {
			let mut f = v as f32;
			while f as f64 > v as f64 {
				f = f32_sub_eps(f);
			}
			RangeItem::FloatValue(f)
		}

		fn upper_trunc(v: i32) -> RangeItem {
			let mut f = v as f32;
			while (f as f64) < (v as f64) {
				f = f32_add_eps(f);
			}
			RangeItem::FloatValue(f)
		}

		let result = match (&self.lower, &self.upper) {
			(RangeItem::IntValue(v), upper) => Range {
				lower: lower_trunc(*v),
				upper: upper.clone(),
			},
			(lower, upper) => Range {
				lower: lower.clone(),
				upper: upper.clone(),
			},
		};

		match (result.lower, result.upper) {
			(lower, RangeItem::IntValue(v)) => Range {
				lower,
				upper: upper_trunc(v),
			},
			(lower, upper) => Range { lower, upper },
		}
	}

	pub fn div(&self, other: &Range) -> Range {
		match (self.positive(), other.positive()) {
			(_, None) => Range::inf(),
			(Some(p1), Some(p2)) => {
				let (numerator_low, numerator_high) = if p2 {
					(&self.lower, &self.upper)
				} else {
					(&self.upper, &self.lower)
				};
				let (deominator_high, deominator_low) = if p1 {
					(&other.lower, &other.upper)
				} else {
					(&other.upper, &other.lower)
				};

				let low =
					numerator_low.div(deominator_low).unwrap_or(RangeItem::NegInf);
				let high =
					numerator_high.div(deominator_high).unwrap_or(RangeItem::PosInf);
				Range::from_items(low, high)
			}
			(None, Some(p2)) => {
				let (numerator_low, numerator_high, deominator) = if p2 {
					(&self.lower, &self.upper, &other.lower)
				} else {
					(&self.upper, &self.lower, &other.upper)
				};
				let low = numerator_low.div(deominator).unwrap_or(RangeItem::NegInf);
				let high = numerator_high.div(deominator).unwrap_or(RangeItem::PosInf);
				Range::from_items(low, high)
			}
		}
	}

	pub fn rem(&self, other: &Range) -> Range {
		if self.is_float() || other.is_float() {
			return Range::inf();
		}

		if other.positive().is_none() {
			return Range::inf();
		}

		let numerator_pos =
			Range::from_items(RangeItem::IntValue(0), RangeItem::PosInf);
		let numerator_neg =
			Range::from_items(RangeItem::NegInf, RangeItem::IntValue(0));

		let numerator_neg = numerator_neg.intersection(self);
		let numerator_pos = numerator_pos.intersection(self);

		let deominator_pos =
			Range::from_items(RangeItem::IntValue(0), RangeItem::PosInf);
		let deominator_neg =
			Range::from_items(RangeItem::NegInf, RangeItem::IntValue(0));

		let deominator_neg = deominator_neg.intersection(other);
		let deominator_pos = deominator_pos.intersection(other);

		let deominator_neg = Range::fromi32(0).sub(&deominator_neg);
		let deominator = deominator_neg.union(&deominator_pos);

		let get_lower = |r: &Range| match &r.lower {
			RangeItem::IntValue(v) => Some(*v),
			_ => None,
		};
		let get_upper = |r: &Range| match &r.upper {
			RangeItem::IntValue(v) => Some(*v),
			_ => None,
		};

		let mut result = Range::contra();

		if !numerator_pos.is_contra() {
			if let (Some(min_a), Some(min_b)) =
				(get_lower(&numerator_pos), get_lower(&deominator))
			{
				result = result.union(&process_rem(
					min_a,
					get_upper(&numerator_pos),
					min_b,
					get_upper(&deominator),
				));
			}
		}

		let neg_numerator_neg = Range::fromi32(0).sub(&numerator_neg);

		if !neg_numerator_neg.is_contra() {
			if let (Some(min_a), Some(min_b)) =
				(get_lower(&neg_numerator_neg), get_lower(&deominator))
			{
				let result_neg = &process_rem(
					min_a,
					get_upper(&neg_numerator_neg),
					min_b,
					get_upper(&deominator),
				);
				let neg_result_neg = Range::fromi32(0).sub(result_neg);
				result = result.union(&neg_result_neg);
			}
		}

		result
	}
}

#[cfg(test)]

mod tests {
	use super::*;

	#[test]
	fn test() {
		let a = Range::fromf32(3.0);
		let b = Range::lowerf32(2.325);
		dbg!(a.intersection(&b));
		dbg!(a.union(&b));
		dbg!(Range::fromf32(0.0).sub(&b));

		let a =
			Range::from_items(RangeItem::IntValue(823), RangeItem::IntValue(824));
		let b = Range::loweri32(23);

		assert_eq!(format!("{:?}", a.add(&b)), "[846,+Inf]");
		assert_eq!(format!("{:?}", b.add(&a)), "[846,+Inf]");
		assert_eq!(format!("{:?}", a.sub(&b)), "[-Inf,801]");
		assert_eq!(format!("{:?}", b.sub(&a)), "[-801,+Inf]");
		assert_eq!(format!("{:?}", a.mul(&b)), "[18929,+Inf]");
		assert_eq!(format!("{:?}", b.mul(&a)), "[18929,+Inf]");
		assert_eq!(format!("{:?}", a.div(&b)), "[0,35]");
		assert_eq!(format!("{:?}", b.div(&a)), "[0,+Inf]");
		assert_eq!(format!("{:?}", a.rem(&b)), "[0,824]");
		assert_eq!(format!("{:?}", b.rem(&a)), "[0,823]");
	}

	fn traverse(a: &Range, b: &Range) -> Vec<String> {
		vec![
			format!("{:?}", a.intersection(&b)).to_string(),
			format!("{:?}", b.intersection(&a)).to_string(),
			format!("{:?}", b.union(&a)).to_string(),
			format!("{:?}", a.union(&b)).to_string(),
			format!("{:?}", a.add(&b)).to_string(),
			format!("{:?}", b.add(&a)).to_string(),
			format!("{:?}", a.sub(&b)).to_string(),
			format!("{:?}", b.sub(&a)).to_string(),
			format!("{:?}", a.mul(&b)).to_string(),
			format!("{:?}", b.mul(&a)).to_string(),
			format!("{:?}", a.div(&b)).to_string(),
			format!("{:?}", b.div(&a)).to_string(),
			format!("{:?}", a.rem(&b)).to_string(),
			format!("{:?}", b.rem(&a)).to_string(),
		]
	}

	#[test]
	fn traverse_arith() {
		let a = Range {
			lower: RangeItem::IntValue(-234),
			upper: RangeItem::IntValue(23),
		};
		let b = Range {
			lower: RangeItem::IntValue(10),
			upper: RangeItem::IntValue(40),
		};
		dbg!(traverse(&a, &b));
		let a = Range {
			lower: RangeItem::IntValue(23),
			upper: RangeItem::IntValue(203),
		};
		let b = Range {
			lower: RangeItem::IntValue(-40),
			upper: RangeItem::IntValue(-10),
		};
		dbg!(traverse(&a, &b));
		let b = Range {
			lower: RangeItem::NegInf,
			upper: RangeItem::IntValue(-10),
		};
		dbg!(traverse(&a, &b));
		let b = Range {
			lower: RangeItem::NegInf,
			upper: RangeItem::IntValue(100),
		};
		dbg!(traverse(&a, &b));
		let a = Range {
			lower: RangeItem::IntValue(900000000),
			upper: RangeItem::PosInf,
		};
		dbg!(traverse(&a, &b));

		let a = Range {
			lower: RangeItem::FloatValue(-234.23),
			upper: RangeItem::FloatValue(23.3),
		};
		let b = Range {
			lower: RangeItem::FloatValue(10.25),
			upper: RangeItem::FloatValue(40.234),
		};
		dbg!(traverse(&a, &b));
		let a = Range {
			lower: RangeItem::FloatValue(23.32),
			upper: RangeItem::FloatValue(203.35),
		};
		let b = Range {
			lower: RangeItem::FloatValue(-40.235),
			upper: RangeItem::FloatValue(-10.8),
		};
		dbg!(traverse(&a, &b));
		let b = Range {
			lower: RangeItem::NegInf,
			upper: RangeItem::FloatValue(-10.23),
		};
		dbg!(traverse(&a, &b));
		let b = Range {
			lower: RangeItem::NegInf,
			upper: RangeItem::FloatValue(100.912),
		};
		dbg!(traverse(&a, &b));
		let a = Range {
			lower: RangeItem::FloatValue(9E10),
			upper: RangeItem::PosInf,
		};
		dbg!(traverse(&a, &b));
	}
}
