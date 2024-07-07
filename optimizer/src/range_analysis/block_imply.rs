use llvm::LlvmTemp;

use std::{
	collections::{HashMap, HashSet},
	fmt::Debug,
};

#[derive(Clone)]
pub struct BlockImplyCondition {
	pub positive: HashSet<LlvmTemp>,
	pub negative: HashSet<LlvmTemp>,
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

	pub fn substution(
		&mut self,
		sub_temp: &LlvmTemp,
		sub_cond: &BlockImplyCondition,
		positive: bool,
	) {
		if self.self_contradictory || sub_cond.self_contradictory {
			return;
		}
		let (right, wrong) = if positive {
			(&mut self.positive, &mut self.negative)
		} else {
			(&mut self.negative, &mut self.positive)
		};

		wrong.remove(sub_temp);
		if right.remove(sub_temp) {
			self.imply(sub_cond.positive.clone(), sub_cond.negative.clone());
		}
	}

	pub fn imply<I>(&mut self, pos: I, neg: I)
	where
		I: IntoIterator<Item = LlvmTemp>,
	{
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

		for tmp in pos {
			self.self_contradictory |=
				work(tmp.clone(), &mut self.positive, &self.negative);
		}
		for tmp in neg {
			self.self_contradictory |=
				work(tmp.clone(), &mut self.negative, &self.positive);
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

pub fn flip_lnot(
	src: &mut BlockImplyCondition,
	lnot_pos_rev: &HashMap<usize, LlvmTemp>,
	lnot_pos: &HashMap<LlvmTemp, usize>,
	lnot_neg: &HashMap<LlvmTemp, usize>,
) {
	let old_pos = std::mem::take(&mut src.positive);
	let old_neg = std::mem::take(&mut src.negative);

	for item in old_pos {
		if let Some(n) = lnot_pos.get(&item) {
			src.positive.insert(lnot_pos_rev[n].clone());
		} else if let Some(n) = lnot_neg.get(&item) {
			src.negative.insert(lnot_pos_rev[n].clone());
		} else {
			src.positive.insert(item);
		}
	}

	for item in old_neg {
		if let Some(n) = lnot_pos.get(&item) {
			src.negative.insert(lnot_pos_rev[n].clone());
		} else if let Some(n) = lnot_neg.get(&item) {
			src.positive.insert(lnot_pos_rev[n].clone());
		} else {
			src.negative.insert(item);
		}
	}
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
