use itertools::Itertools;
use llvm::{CompOp, LlvmTemp, Value};
use std::collections::HashMap;

use super::{
	addictive_synonym::LlvmTempAddictiveSynonym,
	block_imply::BlockImplyCondition,
	range::{Range, RangeItem},
};
#[derive(Debug, Clone)]
pub struct Constrain {
	pub data: Vec<Range>,
}

fn reverse_op(op: &CompOp, pos: bool) -> Option<CompOp> {
	match (pos, op) {
		(true, CompOp::EQ) => Some(CompOp::EQ),
		(true, CompOp::NE) => Some(CompOp::NE),
		(true, CompOp::SGT) => Some(CompOp::SGT),
		(true, CompOp::SGE) => Some(CompOp::SGE),
		(true, CompOp::SLT) => Some(CompOp::SLT),
		(true, CompOp::SLE) => Some(CompOp::SLE),

		(false, CompOp::EQ) => Some(CompOp::NE),
		(false, CompOp::NE) => Some(CompOp::EQ),
		(false, CompOp::SGT) => Some(CompOp::SLE),
		(false, CompOp::SGE) => Some(CompOp::SLT),
		(false, CompOp::SLT) => Some(CompOp::SGE),
		(false, CompOp::SLE) => Some(CompOp::SGT),
		_ => None,
	}
}

fn flip_op(op: Option<CompOp>) -> Option<CompOp> {
	op.map(|op| match op {
		CompOp::EQ => Some(CompOp::EQ),
		CompOp::NE => Some(CompOp::NE),
		CompOp::SGT => Some(CompOp::SLT),
		CompOp::SGE => Some(CompOp::SLE),
		CompOp::SLT => Some(CompOp::SGT),
		CompOp::SLE => Some(CompOp::SGE),
		_ => None,
	})?
}

fn equal(v: &Value, block_id: i32) -> Option<RangeItem> {
	match v {
		Value::Int(i) => Some(RangeItem::IntValue(*i)),
		Value::Float(f) => Some(RangeItem::FloatValue(*f)),
		Value::Temp(v) => match v.var_type {
			llvm::VarType::I32 => Some(RangeItem::IntFuture(v.clone(), block_id, 0)),
			llvm::VarType::F32 => {
				Some(RangeItem::FloatFuture(v.clone(), block_id, 0f32))
			}
			_ => None,
		},
	}
}

fn over(v: &Value, block_id: i32) -> Option<RangeItem> {
	match v {
		Value::Int(i) => Some(RangeItem::IntValue(*i + 1)),
		Value::Float(_f) => None, // TODO
		Value::Temp(v) => match v.var_type {
			llvm::VarType::I32 => Some(RangeItem::IntFuture(v.clone(), block_id, 1)),
			llvm::VarType::F32 => None,
			_ => None,
		},
	}
}

fn under(v: &Value, block_id: i32) -> Option<RangeItem> {
	match v {
		Value::Int(i) => Some(RangeItem::IntValue(*i - 1)),
		Value::Float(_f) => None, // TODO
		Value::Temp(v) => match v.var_type {
			llvm::VarType::I32 => Some(RangeItem::IntFuture(v.clone(), block_id, -1)),
			llvm::VarType::F32 => None,
			_ => None,
		},
	}
}

fn calc_item(
	that: &Value,
	op: &CompOp,
	block_id: i32,
	offset: Value,
) -> (Option<RangeItem>, Option<RangeItem>) {
	let (l, u) = match op {
		CompOp::EQ => (equal(that, block_id), equal(that, block_id)),
		CompOp::NE => (None, None),
		CompOp::SGT => (over(that, block_id), None),
		CompOp::SGE => (equal(that, block_id), None),
		CompOp::SLT => (None, under(that, block_id)),
		CompOp::SLE => (None, equal(that, block_id)),
		_ => (None, None),
	};

	fn add_offset(
		range_item: Option<RangeItem>,
		offset: &Value,
	) -> Option<RangeItem> {
		match (range_item, offset) {
			(Some(RangeItem::IntValue(i)), Value::Int(offset)) => {
				Some(RangeItem::IntValue(i + *offset))
			}
			(Some(RangeItem::FloatValue(f)), Value::Float(offset)) => {
				Some(RangeItem::FloatValue(f + *offset))
			}
			(Some(RangeItem::IntFuture(t, id, i)), Value::Int(offset)) => {
				Some(RangeItem::IntFuture(t, id, i + *offset))
			}
			(Some(RangeItem::FloatFuture(t, id, f)), Value::Float(offset)) => {
				Some(RangeItem::FloatFuture(t, id, f + *offset))
			}
			_ => None,
		}
	}

	(add_offset(l, &offset), add_offset(u, &offset))
}

fn get_range_item(
	tmp: &LlvmTemp,
	l: &Value,
	r: &Value,
	op: &CompOp,
	pos: bool,
	block_id: i32,
	addictive_syn: &LlvmTempAddictiveSynonym,
) -> (Option<RangeItem>, Option<RangeItem>) {
	// dbg!(tmp, l, r, op, pos);

	let op = reverse_op(op, pos); // retrurn None for any op that is not supported here.

	let get_offset = |a: &Value, tmp| match &a {
		Value::Temp(a) => addictive_syn.look_up_offset(a, tmp),
		_ => None,
	};

	if let Some((offset, other, Some(op))) =
		if let Some(offset) = get_offset(l, tmp) {
			Some((offset, r, op))
		} else {
			get_offset(r, tmp).map(|offset| (offset, l, flip_op(op)))
		} {
		calc_item(other, &op, block_id, offset)
	} else {
		(None, None)
	}
}

impl Constrain {
	pub fn build(
		tmp: &LlvmTemp,
		necessary: &BlockImplyCondition,
		_all: &BlockImplyCondition, // TODO: use these information
		comparisons: &HashMap<LlvmTemp, (Value, Value, CompOp, i32)>,
		addicative_syn: &LlvmTempAddictiveSynonym,
	) -> Option<Self> {
		let mut lower: Vec<RangeItem> = vec![];
		let mut upper: Vec<RangeItem> = vec![];

		for item in &necessary.positive {
			if let Some((l, r, op, block_id)) = comparisons.get(item) {
				let (l, u) =
					get_range_item(tmp, l, r, op, true, *block_id, addicative_syn);
				if let Some(l) = l {
					lower.push(l)
				}
				if let Some(u) = u {
					upper.push(u)
				}
			}
		}

		for item in &necessary.negative {
			if let Some((l, r, op, block_id)) = comparisons.get(item) {
				let (l, u) =
					get_range_item(tmp, l, r, op, false, *block_id, addicative_syn);
				if let Some(l) = l {
					lower.push(l)
				}
				if let Some(u) = u {
					upper.push(u)
				}
			}
		}

		let mut result = vec![];

		for pair in lower.into_iter().zip_longest(upper.into_iter()) {
			match pair {
				itertools::EitherOrBoth::Both(l, u) => result.push(Range {
					lower: Some(l),
					upper: Some(u),
				}),
				itertools::EitherOrBoth::Left(l) => result.push(Range {
					lower: Some(l),
					upper: None,
				}),
				itertools::EitherOrBoth::Right(u) => result.push(Range {
					lower: None,
					upper: Some(u),
				}),
			}
		}

		if result.is_empty() {
			return None;
		}

		Some(Self { data: result })
	}
}
