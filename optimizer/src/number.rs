use std::ops::Add;

use rand::{rngs::StdRng, Rng};
use utils::GVN_EVAL_NUMBER;

#[derive(PartialEq, Eq, Debug, Hash, Clone)]
pub struct Number {
	pub value: Vec<u32>,
}

impl Number {
	pub fn new(rng: &mut StdRng) -> Self {
		Number {
			value: (0..GVN_EVAL_NUMBER).map(|_| rng.gen()).collect(),
		}
	}
	pub fn get_base(&self) -> Self {
		let v0 = self.value.first().unwrap();
		Self {
			value: self.value.iter().map(|v| v.wrapping_sub(*v0)).collect(),
		}
	}
	pub fn from(value: impl Into<u32>) -> Self {
		Number {
			value: vec![value.into(); GVN_EVAL_NUMBER],
		}
	}
	pub fn same_value(&self) -> Option<u32> {
		let mut iter = self.value.iter();
		let first = iter.next()?;
		if iter.all(|x| x == first) {
			Some(*first)
		} else {
			None
		}
	}
	pub fn add(x: &Number, y: &Number) -> Number {
		Number {
			value: x
				.value
				.iter()
				.zip(y.value.iter())
				.map(|(x, y)| x.wrapping_add(*y))
				.collect(),
		}
	}
	pub fn sub(x: &Number, y: &Number) -> Number {
		Number {
			value: x
				.value
				.iter()
				.zip(y.value.iter())
				.map(|(x, y)| x.wrapping_sub(*y))
				.collect(),
		}
	}
}

impl<T: Into<u32>> From<T> for Number {
	fn from(value: T) -> Self {
		Number::from(value)
	}
}

impl AsRef<Number> for Number {
	fn as_ref(&self) -> &Number {
		self
	}
}

impl<T: AsRef<Number>> Add<T> for Number {
	type Output = Number;
	fn add(self, rhs: T) -> Self::Output {
		Number::add(&self, rhs.as_ref())
	}
}

impl<T: AsRef<Number>> Add<T> for &Number {
	type Output = Number;
	fn add(self, rhs: T) -> Self::Output {
		Number::add(self, rhs.as_ref())
	}
}
