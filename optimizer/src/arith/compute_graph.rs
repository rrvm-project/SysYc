#![allow(clippy::comparison_chain)]

use llvm::{LlvmTemp, Value, VarType};
use std::{collections::HashSet, fmt::Write};

#[derive(Debug)]
struct OverflowError {}

const MAX_SIZE: usize = 64;

#[derive(PartialEq, Eq, Clone)]
pub enum Single {
	Int(i32),
	Temp(LlvmTemp),
}
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum GraphOp {
	Plus,
	Mul,
}

impl GraphOp {
	fn eval(&self, x1: i32, x2: i32) -> Result<i32, OverflowError> {
		match self {
			GraphOp::Mul => x1.checked_mul(x2).ok_or(OverflowError {}),
			GraphOp::Plus => x1.checked_add(x2).ok_or(OverflowError {}),
		}
	}
}

impl std::fmt::Debug for GraphOp {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Plus => f.write_str("+"),
			Self::Mul => f.write_str("*"),
		}
	}
}

impl std::fmt::Debug for Single {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Int(arg0) => f.write_str(format!(" {} ", arg0).as_str()),
			Self::Temp(arg0) => f.write_str(format!(" {:?} ", arg0).as_str()),
		}
	}
}

impl Ord for Single {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		match (self, other) {
			(Single::Int(_), Single::Temp(_)) => std::cmp::Ordering::Less,
			(Single::Temp(_), Single::Int(_)) => std::cmp::Ordering::Greater,
			(Single::Int(a), Single::Int(b)) => a.cmp(b),
			(Single::Temp(a), Single::Temp(b)) => a.name.cmp(&b.name),
		}
	}
}

impl PartialOrd for Single {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

#[derive(PartialEq, Eq, Clone)]
pub enum GraphValue {
	Single(Single),
	NonTrival((GraphOp, Vec<GraphValue>)),
}

impl GraphValue {
	pub fn as_number(&self) -> Option<i32> {
		match self {
			GraphValue::Single(Single::Int(i)) => Some(*i),
			_ => None,
		}
	}

	pub fn as_tmp(&self) -> Option<&LlvmTemp> {
		match self {
			GraphValue::Single(Single::Temp(t)) => Some(t),
			_ => None,
		}
	}

	pub fn as_single(&self) -> Option<&Single> {
		match self {
			GraphValue::Single(s) => Some(s),
			_ => None,
		}
	}

	fn as_non_trival(&self, op_target: GraphOp) -> Option<&Vec<GraphValue>> {
		match self {
			GraphValue::NonTrival((op, v)) if *op == op_target => Some(v),
			_ => None,
		}
	}

	fn as_non_trival_mut(
		&mut self,
		op_target: GraphOp,
	) -> Option<&mut Vec<GraphValue>> {
		match self {
			GraphValue::NonTrival((op, v)) if *op == op_target => Some(v),
			_ => None,
		}
	}

	pub fn contains_temp(
		&self,
		tmp: &HashSet<LlvmTemp>,
		result: &mut HashSet<LlvmTemp>,
	) {
		match self {
			GraphValue::Single(Single::Temp(t)) => {
				if tmp.contains(t) {
					result.insert(t.clone());
				}
			}
			GraphValue::NonTrival((_, v)) => {
				v.iter().for_each(|item| item.contains_temp(tmp, result));
			}
			_ => {}
		}
	}
}

struct GraphValueCollectIterator<'a> {
	index: usize,
	op: GraphOp,
	my_struct: &'a GraphValue,
}

impl<'a> GraphValue {
	fn collect(&'a self, op: GraphOp) -> GraphValueCollectIterator<'a> {
		GraphValueCollectIterator {
			index: 0,
			my_struct: self,
			op,
		}
	}
}

impl<'a> Iterator for GraphValueCollectIterator<'a> {
	type Item = &'a GraphValue;
	fn next(&mut self) -> Option<Self::Item> {
		let result = self.peek();
		self.index += 1;
		result
	}
}

impl<'a> GraphValueCollectIterator<'a> {
	fn peek(&mut self) -> Option<&'a GraphValue> {
		match self.my_struct {
			GraphValue::NonTrival((op, v)) if *op == self.op || v.len() < 2 => {
				v.get(self.index)
			}
			_ => {
				if self.index == 0 {
					Some(self.my_struct)
				} else {
					None
				}
			}
		}
	}
}

impl GraphValue {
	pub fn size(&self) -> usize {
		match self {
			GraphValue::Single(_) => 1,
			GraphValue::NonTrival((_, graph)) => {
				graph.iter().map(GraphValue::size).sum()
			}
		}
	}

	fn metric(&self) -> usize {
		match self {
			GraphValue::Single(_) => 1,
			GraphValue::NonTrival((_, graph)) => {
				10 + graph.iter().map(GraphValue::metric).sum::<usize>()
			}
		}
	}
}

impl std::fmt::Debug for GraphValue {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Single(s) => f.write_str(format!("{:?}", s).as_str()),
			Self::NonTrival((op, v)) => {
				let left_brace = if *op == GraphOp::Mul { '[' } else { '(' };
				let right_brace = if *op == GraphOp::Mul { ']' } else { ')' };
				f.write_char(left_brace).unwrap();
				v.iter().enumerate().for_each(|(i, v)| {
					if i > 0 {
						f.write_str(format!("{:?}", op).as_str()).unwrap();
					}
					f.write_str(format!("{:?}", v).as_str()).unwrap();
				});
				f.write_char(right_brace)
			}
		}
	}
}

impl Ord for GraphValue {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		match (self, other) {
			(GraphValue::Single(_), GraphValue::NonTrival(_)) => {
				std::cmp::Ordering::Less
			}
			(GraphValue::NonTrival(_), GraphValue::Single(_)) => {
				std::cmp::Ordering::Greater
			}
			(GraphValue::Single(a), GraphValue::Single(b)) => a.cmp(b),
			(GraphValue::NonTrival((opa, va)), GraphValue::NonTrival((opb, vb))) => {
				match opa.cmp(opb) {
					std::cmp::Ordering::Equal => va
						.iter()
						.zip(vb.iter())
						.try_fold(va.len().cmp(&vb.len()), |last, (a, b)| match last {
							std::cmp::Ordering::Equal => Ok(a.cmp(b)),
							_ => Err(last), //Err相当于Break!
						})
						.err()
						.unwrap_or(std::cmp::Ordering::Equal),
					other => other,
				}
			}
		}
	}
}

impl PartialOrd for GraphValue {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

fn gcd(this: &GraphValue, other: &GraphValue) -> Option<GraphValue> {
	// assert the values are sorted
	let mut left = this.collect(GraphOp::Mul);
	let mut right = other.collect(GraphOp::Mul);

	let mut result = vec![];

	while let (Some(a), Some(b)) = (left.peek(), right.peek()) {
		if a == b {
			if a.as_number().is_none() {
				result.push(a.clone());
			}
			left.next();
			right.next();
		} else if a < b {
			left.next();
		} else {
			right.next();
		}
	}

	if result.is_empty() {
		None
	} else {
		Some(GraphValue::NonTrival((GraphOp::Mul, result)))
	}
}

pub fn div(this: &GraphValue, other: &GraphValue) -> Option<GraphValue> {
	let mut left = this.collect(GraphOp::Mul);
	let mut right = other.collect(GraphOp::Mul);

	let mut result = vec![];

	let mut const_part = 1i32;

	while let Some(GraphValue::Single(Single::Int(i))) = left.peek() {
		left.next();
		const_part = const_part.checked_mul(*i)?;
	}

	if const_part == 0 {
		return GraphValue::Single(Single::Int(0)).into();
	}

	while let Some(GraphValue::Single(Single::Int(i))) = right.peek() {
		right.next();
		if *i == 0 {
			return None;
		}
		let quotient = const_part.checked_div(*i)?;
		if quotient.checked_mul(*i)? == const_part {
			const_part = quotient;
		} else {
			return None;
		}
	}

	while let (Some(a), Some(b)) = (left.peek(), right.peek()) {
		if a == b {
			left.next();
			right.next();
		} else if a < b {
			result.push(left.next().unwrap().clone());
		} else if a > b {
			return None;
		} else {
			unreachable!();
		}
	}

	if const_part != 1 {
		result.push(GraphValue::Single(Single::Int(const_part)));
	}

	if right.next().is_some() {
		return None;
	}

	for v in left {
		result.push(v.clone());
	}

	if result.is_empty() {
		GraphValue::Single(Single::Int(1)).into()
	} else {
		GraphValue::NonTrival((GraphOp::Mul, result)).into()
	}
}

pub fn remove_addicative_common(
	left: &GraphValue,
	right: &GraphValue,
) -> Option<(GraphValue, GraphValue)> {
	// dbg!(left, right);

	fn get_not_common<'a, 'b>(
		value1: Vec<&'a GraphValue>,
		value2: Vec<&'b GraphValue>,
	) -> (Vec<&'a GraphValue>, Vec<&'b GraphValue>) {
		let mut iter_left = value1.into_iter().peekable();
		let mut iter_right = value2.into_iter().peekable();
		let mut result_left = vec![];
		let mut result_right = vec![];

		while let (Some(&left), Some(&right)) =
			(iter_left.peek(), iter_right.peek())
		{
			if left == right {
				iter_left.next();
				iter_right.next();
			} else if left < right {
				result_left.push(iter_left.next().unwrap());
			} else {
				result_right.push(iter_right.next().unwrap());
			}
		}

		for v in iter_left {
			result_left.push(v);
		}

		for v in iter_right {
			result_right.push(v);
		}

		(result_left, result_right)
	}

	fn resolve_value(adds: Vec<&GraphValue>) -> GraphValue {
		match adds.len() {
			0 => GraphValue::Single(Single::Int(0)),
			#[allow(suspicious_double_ref_op)]
			1 => adds.first().unwrap().clone().clone(),
			_ => GraphValue::NonTrival((
				GraphOp::Plus,
				adds.into_iter().cloned().collect(),
			)),
		}
	}

	match (left, right) {
		(
			GraphValue::Single(single),
			GraphValue::NonTrival((GraphOp::Plus, adds)),
		) => {
			// dbg!(single, adds);
			let left = GraphValue::Single(single.clone());
			let left_values = vec![&left];
			let right_values: Vec<_> = adds.iter().collect();
			let (result_left, result_right) =
				get_not_common(left_values, right_values);
			if result_right.len() < adds.len() && result_left.is_empty() {
				Some((resolve_value(result_left), resolve_value(result_right)))
			} else {
				None
			}
		}
		(
			GraphValue::NonTrival((GraphOp::Plus, adds)),
			GraphValue::Single(single),
		) => {
			// dbg!(adds, single);
			let right = GraphValue::Single(single.clone());
			let right_values = vec![&right];
			let left_values: Vec<_> = adds.iter().collect();
			let (result_left, result_right) =
				get_not_common(left_values, right_values);
			if result_left.len() < adds.len() && result_right.is_empty() {
				Some((resolve_value(result_left), resolve_value(result_right)))
			} else {
				None
			}
		}
		(
			GraphValue::NonTrival((GraphOp::Plus, adds_l)),
			GraphValue::NonTrival((GraphOp::Plus, adds_r)),
		) => {
			// dbg!(adds_l, adds_r);
			let left_values: Vec<_> = adds_l.iter().collect();
			let right_values: Vec<_> = adds_r.iter().collect();
			let (result_left, result_right) =
				get_not_common(left_values, right_values);
			if result_right.len() < adds_r.len() && result_left.len() < adds_l.len() {
				Some((resolve_value(result_left), resolve_value(result_right)))
			} else {
				None
			}
		}
		_ => None,
	}
}

fn solve_constant(v: &mut GraphValue) -> Result<bool, OverflowError> {
	let old_size = v.size();

	match v {
		GraphValue::Single(_) => return Ok(false),
		GraphValue::NonTrival((op, v)) => {
			let start = match op {
				GraphOp::Mul => 1i32,
				GraphOp::Plus => 0i32,
			};

			let mut value = start;

			let mut new_vec = vec![];

			while let Some(v) = v.pop() {
				if let Some(i) = v.as_number() {
					value = op.eval(value, i)?;
				} else {
					new_vec.push(v);
				}
			}

			if value == 0i32 && *op == GraphOp::Mul {
				new_vec.clear();
			}

			if value != start || new_vec.is_empty() {
				v.push(GraphValue::Single(Single::Int(value)));
			}

			while let Some(item) = new_vec.pop() {
				v.push(item);
			}
		}
	}

	Ok(v.size() < old_size)
}

// return the (index, size) of biggest muli part in target.
fn can_add_para(
	v: &[Option<GraphValue>],
	target: &GraphValue,
) -> Option<(usize, usize)> {
	if target.as_single().is_some() {
		return None;
	}

	if let Some(vec) = target.as_non_trival(GraphOp::Mul) {
		let mut max = None;

		for (i, muli_part) in vec.iter().enumerate() {
			if let Some(factors) = muli_part.as_non_trival(GraphOp::Plus) {
				let mut total = v
					.iter()
					.filter_map(|i: &Option<GraphValue>| i.as_ref())
					.filter(|i| i.as_number().is_none())
					.peekable();
				let mut to_find =
					factors.iter().filter(|i| i.as_number().is_none()).peekable();
				while let (Some(a), Some(b)) = (total.peek(), to_find.peek()) {
					if a == b {
						total.next();
						to_find.next();
					} else if a < b {
						total.next();
					} else {
						break;
					}
				}

				if to_find.peek().is_none() {
					let this_size = muli_part.size();
					if let Some((_, max_size)) = max {
						if max_size < this_size {
							max = Some((i, this_size))
						}
					} else {
						max = Some((i, this_size))
					}
				}
			}
		}

		max
	} else {
		None
	}
}

fn add_para(v: &mut Vec<GraphValue>, target: &mut [GraphValue]) {
	v.sort();
	target.sort();
	let new_v = std::mem::take(v);

	let mut left = new_v.into_iter().peekable();
	let mut right = target.iter().peekable();

	let mut const_part_remain = 0i32;
	let mut const_part_in_para = 0i32;
	let mut in_para = vec![];

	while let Some(value) = left.peek() {
		if let Some(value) = value.as_number() {
			const_part_remain += value;
			left.next();
		} else {
			break;
		}
	}

	while let Some(value) = right.peek() {
		if let Some(value) = value.as_number() {
			const_part_remain -= value;
			const_part_in_para += value;
			right.next();
		} else {
			break;
		}
	}

	while let (Some(l), Some(r)) = (left.peek(), right.peek()) {
		if *l == **r {
			in_para.push(left.next().unwrap());
			right.next();
		} else if *l < **r {
			v.push(left.next().unwrap());
		} else {
			unreachable!()
		}
	}

	assert!(right.peek().is_none());

	for l in left {
		v.push(l);
	}

	if const_part_remain != 0 {
		v.push(GraphValue::Single(Single::Int(const_part_remain)));
		// pushing the para to v will definately make it not sorted, so never mind that v is not sorted here.
	}

	let mut para = vec![];

	if const_part_in_para != 0 {
		para.push(GraphValue::Single(Single::Int(const_part_in_para)));
	}

	para.append(&mut in_para);

	v.push(GraphValue::NonTrival((GraphOp::Plus, para)));
	v.sort();
}

fn mul_const(v: &mut GraphValue) -> Result<bool, OverflowError> {
	//sorted

	fn times(
		times: i32,
		vec: Vec<GraphValue>,
	) -> Result<Vec<GraphValue>, OverflowError> {
		let mut ans = vec![];
		for item in vec {
			let processed = match item {
				GraphValue::Single(Single::Int(i)) => GraphValue::Single(Single::Int(
					i.checked_mul(times).ok_or(OverflowError {})?,
				)),

				any_other => GraphValue::NonTrival((
					GraphOp::Mul,
					vec![GraphValue::Single(Single::Int(times)), any_other],
				)),
			};
			ans.push(processed);
		}
		Ok(ans)
	}

	let mut changed = false;

	if let GraphValue::NonTrival((op, parts)) = v {
		parts.sort();
		if *op == GraphOp::Mul
			&& parts.len() == 2
			&& parts.first().is_some_and(|first| first.as_number().is_some())
		{
			let const_part = parts.first().unwrap().as_number().unwrap();
			if const_part != 1 {
				if let GraphValue::NonTrival((GraphOp::Plus, add_parts)) = &parts[1] {
					if let Ok(timed_add_parts) = times(const_part, add_parts.clone()) {
						*op = GraphOp::Plus;
						*parts = timed_add_parts;
					}
				}
			}
		} else if *op == GraphOp::Mul && parts.len() > 2 {
			let mut const_part = 1i32;
			let mut new_parts = vec![];

			for item in std::mem::take(parts) {
				match item {
					GraphValue::Single(Single::Int(i)) => {
						const_part = const_part.checked_mul(i).ok_or(OverflowError {})?;
					}
					GraphValue::Single(Single::Temp(t)) => {
						new_parts.push(GraphValue::Single(Single::Temp(t)));
					}
					GraphValue::NonTrival((GraphOp::Mul, v)) => {
						new_parts.push(GraphValue::NonTrival((GraphOp::Mul, v)));
					}
					GraphValue::NonTrival((GraphOp::Plus, v)) => {
						if const_part != 1i32
							&& v.first().is_some_and(|item| item.as_number().is_some())
						{
							changed = true;
							new_parts.push(GraphValue::NonTrival((
								GraphOp::Plus,
								times(const_part, v)?,
							)));
							const_part = 1i32;
						} else {
							new_parts.push(GraphValue::NonTrival((GraphOp::Plus, v)));
						}
					}
				}
			}
			if const_part != 1 {
				parts.push(GraphValue::Single(Single::Int(const_part)))
			}

			parts.extend(new_parts);
		}
	}

	// if changed {
	// 	dbg!(debug_v);
	// 	dbg!(&v);
	// }

	Ok(changed)
}

fn make_para(v: &mut Vec<GraphValue>) {
	let mut new_vec = vec![];
	for item in std::mem::take(v) {
		new_vec.push(Some(item));
	}

	let mut max: Option<(usize, usize, usize)> = None;
	for i in 0..new_vec.len() {
		let value_i = new_vec[i].take().unwrap();

		if let Some((inner_index, size)) = can_add_para(&new_vec, &value_i) {
			if !max.is_some_and(|(_, _, old_max)| old_max > size) {
				max = Some((i, inner_index, size));
			}
		}

		new_vec[i] = Some(value_i);
	}

	if let Some((value_index, inner_index, _)) = max {
		let mut value_i = new_vec[value_index].take().unwrap();

		if let Some(target) = value_i.as_non_trival_mut(GraphOp::Mul) {
			let mut remain_v: Vec<GraphValue> =
				new_vec.into_iter().flatten().collect();
			// dbg!(&remain_v);
			let inner = target.get_mut(inner_index).unwrap();
			if let Some(inner) = inner.as_non_trival_mut(GraphOp::Plus) {
				add_para(&mut remain_v, inner);
			}

			remain_v.push(value_i);

			remain_v.sort();
			// dbg!(&remain_v);
			*v = remain_v;
			return;
		}
		unreachable!();
	} else {
		new_vec.into_iter().for_each(|item| v.push(item.unwrap()));
	}
}

fn distributive_law(v: &mut Vec<GraphValue>) -> bool {
	let mut result = vec![];
	let mut changed = false;
	while let Some(new) = v.pop() {
		let mut pending: Option<(GraphValue, Vec<GraphValue>)> = None;
		for item in std::mem::take(&mut result) {
			pending = match (gcd(&new, &item), pending) {
				(None, pending) => {
					result.push(item);
					pending
				}
				(Some(g), None) => Some((g, vec![item])),
				(Some(new_g), Some((old_g, mut old_vec))) => {
					dbg!(&new_g, &old_g, &old_vec);
					if new_g < old_g {
						result.push(item);
						Some((old_g, old_vec))
					} else if new_g == old_g {
						old_vec.push(item);
						Some((old_g, old_vec))
					} else {
						result.append(&mut old_vec);
						Some((new_g, vec![item]))
					}
				}
			};
		}

		if let Some((mut g, mut items)) = pending {
			g.sort();
			let mut remains = vec![];
			items.push(new);

			for item in items {
				let remain = div(&item, &g).unwrap();
				remains.push(remain);
			}

			let mut product_vec: Vec<GraphValue> =
				g.collect(GraphOp::Mul).cloned().collect();
			product_vec.push(GraphValue::NonTrival((GraphOp::Plus, remains)));

			changed = true;
			product_vec.sort();
			result.push(GraphValue::NonTrival((GraphOp::Mul, product_vec)));
		} else {
			result.push(new);
		}
	}

	result.sort();
	*v = result;
	changed
}

impl GraphValue {
	fn sort(&mut self) {
		match self {
			GraphValue::Single(_) => {}
			GraphValue::NonTrival((_, v)) => {
				v.iter_mut().for_each(GraphValue::sort);
				v.sort();
			}
		}
	}

	fn collect_with_op(self, output: &mut Vec<GraphValue>, op: GraphOp) {
		match self {
			GraphValue::NonTrival((op_this, v)) => {
				if op_this == op || v.len() < 2 {
					for item in v {
						item.collect_with_op(output, op);
					}
				} else {
					output.push(GraphValue::NonTrival((op_this, v)));
				}
			}
			_ => {
				output.push(self);
			}
		}
	}

	fn reduce(&mut self) {
		let mut reduce_self = None;
		match self {
			GraphValue::Single(_) => {}
			GraphValue::NonTrival((op, v)) => {
				v.iter_mut().for_each(GraphValue::reduce);

				let mut new_v = vec![];
				std::mem::take(v).into_iter().for_each({
					|value| {
						value.collect_with_op(&mut new_v, *op);
					}
				});

				if new_v.len() == 1 {
					reduce_self = new_v.pop();
				} else {
					*v = new_v;
				}
			}
		}

		if let Some(reduce_self) = reduce_self {
			*self = reduce_self;
		}
	}

	fn simplify(&mut self) -> Result<bool, OverflowError> {
		let mut changed = false;

		match self {
			GraphValue::Single(_) => {}
			GraphValue::NonTrival((op, v)) => {
				for item in v.iter_mut() {
					changed |= item.simplify()?;
				}
				if *op == GraphOp::Plus {
					make_para(v);
					changed |= distributive_law(v);
				}
			}
		}
		changed |= solve_constant(self)?;
		changed |= mul_const(self)?;
		Ok(changed)
	}

	fn sanity(&mut self) -> Result<(), OverflowError> {
		let mut cnt = 0;
		loop {
			self.sort();
			self.reduce();

			let old = self.metric();
			let mut changed = false;
			changed |= self.simplify()?;
			self.reduce();
			let new = self.metric();

			cnt += 1;
			if cnt > 10 && (!changed || old == new) {
				break;
			}
		}
		Ok(())
	}

	pub fn check_over_size(self) -> Option<Self> {
		if self.size() <= MAX_SIZE {
			Some(self)
		} else {
			None
		}
	}

	pub fn add(&self, other: &GraphValue) -> Option<GraphValue> {
		let mut result =
			GraphValue::NonTrival((GraphOp::Plus, vec![self.clone(), other.clone()]));

		result.sanity().ok()?;

		result.check_over_size()
	}

	pub fn sub(&self, other: &GraphValue) -> Option<GraphValue> {
		let mut result = GraphValue::NonTrival((
			GraphOp::Plus,
			vec![
				self.clone(),
				GraphValue::NonTrival((
					GraphOp::Mul,
					vec![GraphValue::Single(Single::Int(-1)), other.clone()],
				)),
			],
		));

		result.sanity().ok()?;
		result.check_over_size()
	}

	pub fn mul(&self, other: &GraphValue) -> Option<GraphValue> {
		let mut result =
			GraphValue::NonTrival((GraphOp::Mul, vec![self.clone(), other.clone()]));

		result.sanity().ok()?;

		result.check_over_size()
	}

	pub fn div(&self, other: &GraphValue) -> Option<GraphValue> {
		if let Some(mut result) = div(self, other) {
			result.sanity().ok()?;
			Some(result)
		} else {
			None
		}
	}

	pub fn from_value(value: Value) -> Option<GraphValue> {
		match value {
			Value::Int(i) => Some(GraphValue::Single(Single::Int(i))),
			Value::Temp(t) if t.var_type == VarType::I32 => {
				Some(GraphValue::Single(Single::Temp(t)))
			}
			_ => None,
		}
	}

	fn subsitute(&mut self, temp: &LlvmTemp, value: &GraphValue) {
		match self {
			GraphValue::Single(Single::Temp(t)) if *t == *temp => {
				*self = value.clone();
			}
			GraphValue::NonTrival((_, v)) => {
				v.iter_mut().for_each(|v| v.subsitute(temp, value))
			}
			_ => {}
		}
	}

	pub fn substitude_checked(
		&self,
		temp: &LlvmTemp,
		value: &GraphValue,
	) -> Option<GraphValue> {
		let mut new = self.clone();
		new.subsitute(temp, value);
		new.sanity().ok()?;
		new.check_over_size()
	}
}

#[cfg(test)]
mod tests {
	use std::vec;

	use super::*; // 导入主模块中的所有内容

	fn get_tmp(id: usize) -> Single {
		Single::Temp(LlvmTemp {
			name: format!("{}", id),
			is_global: false,
			var_type: VarType::I32,
		})
	}

	#[test]
	fn test_distributive() {
		let b = GraphValue::NonTrival((
			GraphOp::Plus,
			vec![
				GraphValue::Single(Single::Int(5)),
				GraphValue::Single(get_tmp(5)),
				GraphValue::NonTrival((
					GraphOp::Mul,
					vec![
						GraphValue::Single(Single::Int(6)),
						GraphValue::Single(get_tmp(6)),
					],
				)),
			],
		));

		let c = GraphValue::NonTrival((
			GraphOp::Plus,
			vec![
				GraphValue::Single(Single::Int(1000)),
				GraphValue::Single(get_tmp(5)),
				GraphValue::NonTrival((
					GraphOp::Mul,
					vec![
						GraphValue::Single(Single::Int(6)),
						GraphValue::Single(get_tmp(6)),
					],
				)),
			],
		));

		let mut a = GraphValue::NonTrival((
			GraphOp::Plus,
			vec![
				GraphValue::NonTrival((
					GraphOp::Mul,
					vec![b.clone(), GraphValue::Single(Single::Int(3))],
				)),
				GraphValue::NonTrival((
					GraphOp::Mul,
					vec![b.clone(), GraphValue::Single(Single::Int(3))],
				)),
				GraphValue::NonTrival((
					GraphOp::Mul,
					vec![b.clone(), GraphValue::Single(get_tmp(33))],
				)),
				b.clone(),
				GraphValue::Single(get_tmp(23423)),
				c.clone(),
			],
		));

		dbg!(&a);

		// a.sort();
		// a.reduce();
		// a.simplify();
		// a.sort();
		// a.reduce();

		// dbg!(&a);

		a.sanity().unwrap();

		dbg!(&a);
	}

	#[test]
	fn test_reduce() {
		let mut a = GraphValue::NonTrival((
			GraphOp::Plus,
			vec![
				GraphValue::Single(Single::Int(-100)),
				GraphValue::Single(get_tmp(3)),
				GraphValue::NonTrival((
					GraphOp::Plus,
					vec![
						GraphValue::Single(Single::Int(4)),
						GraphValue::Single(get_tmp(4)),
						GraphValue::NonTrival((
							GraphOp::Mul,
							vec![
								GraphValue::Single(Single::Int(5)),
								GraphValue::Single(get_tmp(5)),
								GraphValue::NonTrival((
									GraphOp::Plus,
									vec![
										GraphValue::Single(Single::Int(6)),
										GraphValue::Single(get_tmp(6)),
									],
								)),
							],
						)),
						GraphValue::NonTrival((
							GraphOp::Mul,
							vec![
								GraphValue::Single(Single::Int(14)),
								GraphValue::Single(get_tmp(14)),
								GraphValue::NonTrival((
									GraphOp::Mul,
									vec![
										GraphValue::Single(Single::Int(15)),
										GraphValue::Single(get_tmp(15)),
										GraphValue::NonTrival((
											GraphOp::Mul,
											vec![
												GraphValue::Single(Single::Int(16)),
												GraphValue::Single(get_tmp(16)),
											],
										)),
									],
								)),
							],
						)),
						GraphValue::NonTrival((
							GraphOp::Plus,
							vec![
								GraphValue::Single(Single::Int(14)),
								GraphValue::Single(get_tmp(14)),
								GraphValue::NonTrival((
									GraphOp::Plus,
									vec![
										GraphValue::Single(Single::Int(15)),
										GraphValue::Single(get_tmp(15)),
										GraphValue::NonTrival((
											GraphOp::Plus,
											vec![
												GraphValue::Single(Single::Int(16)),
												GraphValue::Single(get_tmp(16)),
											],
										)),
									],
								)),
							],
						)),
					],
				)),
			],
		));

		dbg!(&a);

		a.reduce();

		dbg!(&a);
	}

	#[test]
	fn test_mul_1() {
		let b = GraphValue::NonTrival((
			GraphOp::Plus,
			vec![
				GraphValue::Single(Single::Int(5)),
				GraphValue::Single(get_tmp(5)),
				GraphValue::NonTrival((
					GraphOp::Mul,
					vec![
						GraphValue::Single(Single::Int(6)),
						GraphValue::Single(get_tmp(6)),
					],
				)),
			],
		));

		let c = GraphValue::NonTrival((
			GraphOp::Plus,
			vec![
				GraphValue::Single(Single::Int(82)),
				GraphValue::Single(get_tmp(5)),
			],
		));

		let mut a = GraphValue::NonTrival((GraphOp::Mul, vec![b.clone(), c]));

		a.sanity().unwrap();

		dbg!(&a);

		let c = div(&a, &b);

		dbg!(&c);

		let mut d = GraphValue::NonTrival((
			GraphOp::Mul,
			vec![GraphValue::Single(Single::Int(1))],
		));

		d.sanity().unwrap();

		dbg!(d);
		let mut d = GraphValue::NonTrival((
			GraphOp::Mul,
			vec![GraphValue::Single(Single::Int(0))],
		));

		d.sanity().unwrap();

		dbg!(d);
		let mut d = GraphValue::NonTrival((
			GraphOp::Plus,
			vec![
				GraphValue::Single(Single::Int(80)),
				GraphValue::Single(Single::Int(-80)),
			],
		));

		d.sanity().unwrap();

		dbg!(d);
	}

	#[test]
	fn test_sub() {
		let a = GraphValue::Single(get_tmp(2));
		let b = GraphValue::Single(get_tmp(3));

		let mut h = a.add(&b).unwrap();

		dbg!(&h);
		for _ in 0..5 {
			h = h.add(&h).unwrap();
			dbg!(&h);
		}

		let i = h.mul(&h).unwrap();

		dbg!(&i);

		let i = i.div(&GraphValue::Single(Single::Int(512))).unwrap();
		dbg!(i.div(&a.add(&b).unwrap()));
		dbg!(i.div(&a.add(&b).unwrap()).unwrap().div(&a.add(&b).unwrap()));

		let mut h = a.add(&b).unwrap();

		dbg!(&h);
		for _ in 0..30 {
			h = h.add(&h).unwrap();
			dbg!(&h);
		}

		assert!(h.add(&h).is_none()); // over flow here!

		// cargo test --package optimizer --lib -- arith::compute_graph::tests::test_sub --exact --show-output
	}
}
