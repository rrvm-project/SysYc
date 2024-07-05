use llvm::LlvmTemp;

use std::{collections::HashSet, fmt::Debug};

#[derive(Clone)]
pub struct BlockImplyCondition {
	positive: HashSet<LlvmTemp>,
	negative: HashSet<LlvmTemp>,
	self_contradictory: bool,
}

impl Debug for BlockImplyCondition {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		fn to_str(set: &HashSet<LlvmTemp>) -> String {
			let names: Vec<String> = set.iter().map(|t| t.name.clone()).collect();
			names.join(" ")
		}

		if self.self_contradictory {
			f.debug_struct("BlockImplyCondition")
				.field("self_contradictory", &self.self_contradictory)
				.finish()
		} else {
			f.debug_struct("BlockImplyCondition")
				.field("positive", &to_str(&self.positive))
				.field("negative", &to_str(&self.negative))
				.finish()
		}
	}
}

impl BlockImplyCondition {
	pub fn new() -> Self {
		BlockImplyCondition {
			positive: HashSet::new(),
			negative: HashSet::new(),
			self_contradictory: false,
		}
	}

	fn contradiction() -> Self {
		BlockImplyCondition {
			positive: HashSet::new(),
			negative: HashSet::new(),
			self_contradictory: true,
		}
	}

	pub fn size(&self) -> usize {
		if self.self_contradictory {
			usize::MAX
		} else {
			self.positive.len() + self.negative.len()
		}
	}

	pub fn imply(&mut self, pos: Option<LlvmTemp>, neg: Option<LlvmTemp>) {
		if self.self_contradictory {
			return;
		}
		fn work(
			tmp: LlvmTemp,
			same: &mut HashSet<LlvmTemp>,
			not_same: &HashSet<LlvmTemp>,
		) -> bool {
			if not_same.contains(&tmp) {
				return true;
			}
			if same.contains(&tmp) {
				return false;
			}
			same.insert(tmp);
			false
		}

		if let Some(tmp) = pos {
			self.self_contradictory |= work(tmp, &mut self.positive, &self.negative);
		}
		if let Some(tmp) = neg {
			self.self_contradictory |= work(tmp, &mut self.negative, &self.positive);
		}
		if self.self_contradictory {
			self.positive.clear();
			self.negative.clear();
		}
	}

	pub fn both(&mut self, other: &BlockImplyCondition) {
		if self.self_contradictory {
			*self = other.clone();
		}
		if other.self_contradictory {
			return;
		}
		self.positive.retain(|c| other.positive.contains(c));
		self.negative.retain(|c| other.negative.contains(c));
	}
}

pub fn add_implication(
	src: &BlockImplyCondition,
	pos: Option<LlvmTemp>,
	neg: Option<LlvmTemp>,
) -> BlockImplyCondition {
	let mut result = src.clone();
	result.imply(pos, neg);
	result
}

pub fn general_both<T: IntoIterator<Item = BlockImplyCondition>>(
	iterator: T,
) -> BlockImplyCondition {
	iterator.into_iter().fold(
		BlockImplyCondition::contradiction(),
		|mut c, new| {
			c.both(&new);
			c
		},
	)
}
