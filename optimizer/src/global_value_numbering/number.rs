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
}
