use itertools::Itertools;
use llvm::{
	ArithInstr, ArithOp, ConvertInstr, ConvertOp, LlvmTemp, PhiInstr, Value,
};
use utils::from_label;

use std::{
	collections::{HashMap, HashSet, VecDeque},
	fmt::Debug,
	vec,
};

use utils::errors::Result;

use super::{
	constrain::Constrain,
	range::{Range, RangeItem},
	range_arith::range_calculate,
	tarjan::Graph,
	widen_operator::{SimpleWidenOperator, WidenOp},
};

#[derive(Clone)]
pub struct ConstrainGraph {
	nodes: Vec<Option<Node>>,
	tmp_to_nodes: HashMap<LlvmTemp, HashMap<i32, usize>>,
}

impl std::fmt::Debug for ConstrainGraph{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str("ConstrainGraph:".into())?;
		for (i, node) in self.nodes.iter().enumerate(){
			if let Some(node) = node{
				f.write_fmt(format_args!("{}: prev{:?} next{:?} future{:?} range{:?} inner{:?}\n",i, node.prev, node.next, node.future, node.range, node.inner))?;
			} else {
				f.write_fmt(format_args!("{}: TAKEN\n", i))?;
			}
		}
		Ok(())
	}
}

#[derive(Debug, Clone)]
pub enum NodeInner {
	Temp(LlvmTemp, i32),
	Const(Range),
	Constraint(Range),
	RangePhi(Vec<usize>),
	Op(ArithOp, Vec<usize>),
	Convert(ConvertOp, usize),
	PlaceHolder,
}

#[derive(Debug, Clone)]
pub struct Node {
	id: usize,
	pub next: Vec<usize>,
	pub future: Vec<usize>,
	pub inner: NodeInner,
	pub range: Option<Range>,
	pub prev: Vec<usize>,
}

impl Node {
	pub fn force_solve_future(&mut self) {
		fn solve(a: &mut RangeItem, default: RangeItem) {
			match a {
				RangeItem::IntFuture(_, _, _) => *a = default,
				RangeItem::FloatFuture(_, _, _) => *a = default,
				_ => (),
			}
		}
		if let Some(future) = &mut self.range {
			solve(&mut future.lower, RangeItem::NegInf);
			solve(&mut future.upper, RangeItem::PosInf);
		}
	}
	pub fn solve_future(&mut self, temp: &LlvmTemp, bbid: i32, range: &Range) {
		let solve_one = |future: &RangeItem, new: &RangeItem| match (future, new) {
			(
				RangeItem::IntFuture(future_temp, future_id, offset),
				RangeItem::IntValue(bound),
			) => {
				if *temp == *future_temp && bbid == *future_id {
					RangeItem::IntValue(bound + *offset)
				} else {
					future.clone()
				}
			}
			(
				RangeItem::FloatFuture(future_temp, future_id, offset),
				RangeItem::FloatValue(bound),
			) => {
				if *temp == *future_temp && bbid == *future_id {
					RangeItem::FloatValue(bound + *offset)
				} else {
					future.clone()
				}
			}
			(RangeItem::FloatFuture(_, _, _), RangeItem::PosInf) => RangeItem::PosInf,
			(RangeItem::FloatFuture(_, _, _), RangeItem::NegInf) => RangeItem::NegInf,
			(RangeItem::IntFuture(_, _, _), RangeItem::PosInf) => RangeItem::PosInf,
			(RangeItem::IntFuture(_, _, _), RangeItem::NegInf) => RangeItem::NegInf,
			_ => future.clone(),
		};

		let solve = |future: &Range| -> Range {
			let mut ans = Range::from_items(
				solve_one(&future.lower, &range.lower),
				solve_one(&future.upper, &range.upper),
			);
			ans.contra_check();
			ans
		};

		if let NodeInner::Constraint(c) = &mut self.inner {
			*c = solve(c);
		}
		if let Some(future) = &mut self.range {
			*future = solve(future);
		}
	}
}

impl Graph<'_> for ConstrainGraph {
	fn next(&'_ self, u: usize) -> Box<dyn Iterator<Item = usize> + '_> {
		Box::new(
			self
				.get_node_ref(u)
				.next()
				.iter()
				.cloned()
				.chain(self.get_node_ref(u).future().iter().cloned()),
		)
	}
}

impl ConstrainGraph {
	pub fn grow_analysis(&mut self, scc: &Vec<usize>) {
		let mut work_list = VecDeque::new();
		scc.iter().for_each(|node_id| work_list.push_back(*node_id));
		let mut this_scc = HashSet::new();
		for item in scc {
			this_scc.insert(*item);
		}

		let widen_op = SimpleWidenOperator;
		while let Some(id) = work_list.pop_front() {
			match self.widening_node(id, &widen_op) {
				Err(_) => {
					// println!("failed {} {}", id, e.to_string());
				}
				Ok(true) => {
					// println!("updated {} {:?}", id, self.get_node_ref(id).range);
					for next in self.get_node_ref(id).next() {
						if this_scc.contains(next) {
							work_list.push_back(*next)
						}
					}
				}
				_ => {
					// println!("not updated {}", id);
				}
			}
		}
		for item in scc {
			let node = self.get_node_mut(*item);
			if node.range.is_none() {
				node.range = Some(Range::inf());
			}
		}
	}

	pub fn solve_future(&mut self, scc: &Vec<usize>) {
		for id in scc {
			let mut node = self.take_node(*id).unwrap();
			let mut need_force_solve = false;
			if let (NodeInner::Temp(tmp, bbid), Some(range)) =
				(&node.inner, &node.range)
			{
				for to in node.future() {
					if to == id {
						need_force_solve = true;
					} else {
						self.get_node_mut(*to).solve_future(tmp, *bbid, range)
					}
				}
			}
			if need_force_solve {
				node.force_solve_future();
			}
			self.put_node(*id, node);
		}

		for id in scc {
			self.get_node_mut(*id).force_solve_future();
		}
	}

	pub fn narrowing(&mut self, scc: &[usize]) {
		let mut work_list = VecDeque::new();
		scc.iter().for_each(|node_id| work_list.push_back(*node_id));

		while let Some(id) = work_list.pop_front() {
			if self.narrowing_node(id).unwrap_or(false) {
				for next in self.get_node_ref(id).next() {
					work_list.push_back(*next)
				}
			}
		}
	}

	fn process_node<P>(&mut self, id: usize, process: P) -> Result<bool>
	where
		P: Fn(&mut Node, &Vec<&Option<Range>>) -> Result<bool>,
	{
		let mut node = self.take_node(id).unwrap();
		let data: Vec<&Option<Range>> = if let Some(oprants) = node.oprants() {
			// if the node requries a specific order of data, make it happy,
			oprants.iter()
		} else {
			// else just get the data in increasing order of node.
			node.prev.iter()
		}
		.map(|x| &self.get_node_ref(*x).range)
		.collect_vec();

		let result = process(&mut node, &data);
		self.put_node(id, node);
		result
	}

	pub fn widening_node<'a, T: WidenOp + 'a>(
		&'a mut self,
		id: usize,
		widen_op: &'a T,
	) -> Result<bool> {
		let process = |node: &mut Node, data: &Vec<&Option<Range>>| {
			node.widening(data, widen_op)
		};
		self.process_node(id, process)
	}

	pub fn narrowing_node(&mut self, id: usize) -> Result<bool> {
		self.process_node(id, Node::narrowing)
	}

	fn set_prev(&mut self) {
		// for node in self.nodes.iter_mut() {
		// 	if let Some(node) = node {
		// 		node.prev.clear();
		// 	}
		// }
		for node in self.nodes.iter_mut().flatten() {
			node.prev.clear();
		}

		for i in 0..self.nodes.len() {
			if let Some(node) = self.take_node(i) {
				node
					.next()
					.iter()
					.map(|index| {
						self.get_node_mut(*index).prev.push(i);
					})
					.count();
				self.put_node(i, node);
			}
		}

		for node in self.nodes.iter(){
			if let Some(node) = node{
				match (&node.inner, node.prev.len()) {
					(NodeInner::Temp(t,bbid) ,n) => {
						if n > 1{
							for prev in &node.prev{
								dbg!(&self.nodes[*prev]);
							}
							assert!(n <= 1,"{:?}@{}, {:?}", t, bbid, node.prev)
						}
					}
					_ => ()
				}
			}
		}
	}

	pub fn prepare(&mut self) {
		self.set_prev();
		for node in &mut self.nodes.iter_mut().flatten() {
			match node.inner {
				NodeInner::PlaceHolder => {
					unreachable!("place holder should only be used when constructing")
				}
				NodeInner::Temp(_, _) => {
					if node.prev.is_empty() {
						node.range = Some(Range::inf())
					}
				}
				_ => {}
			}
		}
	}
}

fn extract<'a, T>(input: &Vec<&'a Option<T>>) -> Option<Vec<&'a T>> {
	let mut result = vec![];
	for item in input {
		if let Some(data) = item {
			result.push(data);
		} else {
			return None;
		}
	}

	Some(result)
}

fn extract_nothing<'a, T: Debug>(input: &[&'a Option<T>]) -> Vec<&'a T> {
	let mut result = vec![];
	// for item in input {
	// 	if let Some(data) = item {
	// 		result.push(data);
	// 	}
	// }
	for data in input.iter().copied().flatten() {
		result.push(data);
	}
	result
}

impl Node {
	fn update_range(&mut self, new_range: Option<Range>) -> bool {
		if self.range == new_range {
			false
		} else {
			self.range = new_range;
			true
		}
	}

	fn calculate(&self, data: Vec<&Range>, widening: bool) -> Result<Range> {
		match (&self.inner, data.len()) {
			// (NodeInner::Temp(_, _), 0) => Ok(Range::inf()),
			(NodeInner::Temp(_, _), 1) => Ok(data[0].clone()),
			(NodeInner::Constraint(range), 1) => {
				if widening {
					Ok(data[0].clone())
				} else {
					Ok(range.intersection(data[0]))
				}
			}
			(NodeInner::Const(range), 0) => Ok(range.clone()),

			(NodeInner::RangePhi(srcs), n) => {
				if n == srcs.len() || (n > 0 && widening) {
					Ok(data.into_iter().fold(Range::contra(), |x, y| x.union(y)))
				} else {
					Err(utils::SysycError::SystemError(
						"incorrect length of input range of rangephi for narrowing"
							.to_string(),
					))
				}
			}
			(NodeInner::Op(op, srcs), n) => {
				if n != srcs.len() {
					Err(utils::SysycError::SystemError(
						"incorrect length of input ranges of op ".to_string(),
					))
				} else {
					let range = range_calculate(op, data);
					Ok(range)
				}
			}
			(NodeInner::Convert(op, _), 1) => match op {
				ConvertOp::Int2Float => Ok(data[0].to_float()),
				ConvertOp::Float2Int => Ok(data[0].to_int()),
			},
			_ => Err(utils::SysycError::SystemError(
				"incorrect length of input ranges ".to_string(),
			)),
		}
	}

	pub fn widening<T: WidenOp>(
		&mut self,
		data: &[&Option<Range>],
		widenop: &T,
	) -> Result<bool> {
		let evaluation = self.calculate(extract_nothing(data), true)?;
		Ok(widenop.widen(&mut self.range, evaluation))
	}

	pub fn narrowing(&mut self, data: &Vec<&Option<Range>>) -> Result<bool> {
		if let Some(data) = extract(data) {
			let evaluation = self.calculate(data, false);
			evaluation.map(|eva| self.update_range(Some(eva)))
		} else {
			Err(utils::SysycError::SystemError(
				"option found in input range for narrowing".to_string(),
			))
		}
	}

	pub fn get_id(&self) -> usize {
		self.id
	}

	pub fn get_inner_range_ref(&self) -> Option<&Range> {
		match &self.inner {
			NodeInner::Constraint(c) => Some(c),
			NodeInner::Const(c) => Some(c),
			_ => None,
		}
	}

	pub fn oprants(&self) -> Option<&Vec<usize>> {
		//
		match &self.inner {
			NodeInner::Op(_, oprants) => Some(oprants),
			// Note: for other types, the order of prevs does not matter!
			_ => None,
		}
	}

	pub fn next(&self) -> &Vec<usize> {
		&self.next
	}

	pub fn future(&self) -> &Vec<usize> {
		&self.future
	}

	#[allow(dead_code)]
	pub fn as_phi_node_inner(&mut self) -> Option<&mut Vec<usize>> {
		match &mut self.inner {
			NodeInner::RangePhi(phi) => Some(phi),
			_ => None,
		}
	}
}

impl ConstrainGraph {
	pub fn new() -> Self {
		Self {
			nodes: vec![],
			tmp_to_nodes: HashMap::new(),
		}
	}

	pub fn len(&self) -> usize {
		self.nodes.len()
	}

	fn insert_node(&mut self, inner: NodeInner) -> &mut Node {
		let id = self.nodes.len();
		self.nodes.push(Some(Node {
			id,
			next: vec![],
			future: vec![],
			inner,
			range: None,
			prev: vec![],
		}));

		self.nodes.get_mut(id).unwrap().as_mut().unwrap()
	}

	pub fn look_up_tmp_node(
		&self,
		tmp: &LlvmTemp,
		basicblockid: i32,
	) -> Option<usize> {
		self.tmp_to_nodes.get(tmp).and_then(|x| x.get(&basicblockid).copied())
	}

	pub fn get_tmp_node(
		&mut self,
		tmp: &LlvmTemp,
		basicblockid: i32,
	) -> &mut Node {
		if let Some(id) = self.look_up_tmp_node(tmp, basicblockid) {
			self.nodes.get_mut(id).unwrap().as_mut().unwrap()
		} else {
			let id = self.insert_node(NodeInner::Temp(tmp.clone(), basicblockid)).id;
			self
				.tmp_to_nodes
				.entry(tmp.clone())
				.or_default()
				.insert(basicblockid, id);
			self.get_node_mut(id)
		}
	}

	#[allow(dead_code)]
	pub fn get_node_ref(&self, id: usize) -> &Node {
		self.nodes.get(id).unwrap().as_ref().unwrap()
	}

	pub fn take_node(&mut self, id: usize) -> Option<Node> {
		self.nodes.get_mut(id).unwrap().take()
	}

	pub fn put_node(&mut self, id: usize, t: Node) -> Option<Node> {
		self.nodes.get_mut(id).unwrap().replace(t)
	}

	pub fn get_node_mut(&mut self, id: usize) -> &mut Node {
		self.nodes.get_mut(id).unwrap().as_mut().unwrap()
	}

	pub fn add_future(&mut self, constrain_node_id: usize) {
		fn add_one(this: &mut ConstrainGraph, item: &RangeItem, id: usize) {
			if let Some((t, block)) = match item {
				RangeItem::IntFuture(t, block, _) => Some((t, *block)),
				RangeItem::FloatFuture(t, block, _) => Some((t, *block)),
				_ => None,
			} {
				this.get_tmp_node(t, block).future.push(id)
			}
		}
		let constrain_node = self.take_node(constrain_node_id).unwrap();
		if let Some(range) = constrain_node.get_inner_range_ref() {
			add_one(self, &range.lower, constrain_node_id);
			add_one(self, &range.upper, constrain_node_id);
		}
		self.put_node(constrain_node_id, constrain_node);
	}

	pub fn insert_phi_node(&mut self, dst: usize, srcs: Vec<usize>) {
		if srcs.is_empty() {
			return;
		}

		if srcs.len() == 1 {
			self.get_node_mut(srcs[0]).next.push(dst);
			return;
		}

		let src_iter = srcs.iter();
		let phi_node = self.insert_place_holder_and_link(dst, src_iter);
		phi_node.inner = NodeInner::RangePhi(srcs);
	}

	fn get_srcs<'a, T: IntoIterator<Item = (&'a Value, i32)>>(
		&mut self,
		values: T,
	) -> Vec<usize> {
		values
			.into_iter()
			.map(|(src, src_block_id)| {
				match src {
					llvm::Value::Int(i) => {
						self.insert_node(NodeInner::Const(Range::fromi32(*i)))
					}
					llvm::Value::Float(f) => {
						self.insert_node(NodeInner::Const(Range::fromf32(*f)))
					}
					llvm::Value::Temp(t) => self.get_tmp_node(t, src_block_id),
				}
				.get_id()
			})
			.collect()
	}

	fn insert_place_holder_and_link<'a, T: IntoIterator<Item = &'a usize>>(
		&mut self,
		dst: usize,
		srcs: T,
	) -> &mut Node {
		let place_holder = self.insert_node(NodeInner::PlaceHolder).get_id();
		srcs
			.into_iter()
			.map(|node_id| self.get_node_mut(*node_id).next.push(place_holder))
			.count();
		self.get_node_mut(place_holder).next.push(dst);
		self.get_node_mut(place_holder)
	}

	pub fn handle_arith_instr(&mut self, instr: &ArithInstr, basicblockid: i32) {
		let target = self.get_tmp_node(&instr.target, basicblockid).get_id();
		let srcs = self
			.get_srcs(vec![(&instr.lhs, basicblockid), (&instr.rhs, basicblockid)]);

		let src_iter = srcs.iter();
		let node = self.insert_place_holder_and_link(target, src_iter);
		node.inner = NodeInner::Op(instr.op, srcs);
	}

	pub fn handle_convert_instr(
		&mut self,
		instr: &ConvertInstr,
		basicblockid: i32,
	) {
		let target = self.get_tmp_node(&instr.target, basicblockid).get_id();
		let srcs = self.get_srcs(vec![(&instr.lhs, basicblockid)]);

		let src_iter = srcs.iter();
		let node = self.insert_place_holder_and_link(target, src_iter);
		node.inner = NodeInner::Convert(instr.op, srcs[0]);
	}

	pub fn handle_phi_instr(&mut self, phi: &PhiInstr, basicblockid: i32) {
		let dst = self.get_tmp_node(&phi.target, basicblockid).id;
		let srcs = self
			.get_srcs(phi.source.iter().map(|(src, block)| (src, from_label(block))));
		self.insert_phi_node(dst, srcs);
	}

	pub fn handle_live_in<T: IntoIterator<Item = i32>>(
		&mut self,
		live_outs: T,
		this: LlvmTemp,
		constrain: Option<Constrain>,
		basicblockid: i32,
	) {
		let this_id = self.get_tmp_node(&this, basicblockid).id;

		let live_outs: Vec<usize> = live_outs
			.into_iter()
			.map(|src_bb_id| self.get_tmp_node(&this, src_bb_id).get_id())
			.collect();

		let mut dst = this_id;

		/*
				Src1 ---
						\
						 phi----> constrain ---> constrain --> .... --> this
						/
				Src2 ---
		*/

		if let Some(constrain) = constrain {
			for item in constrain.data {
				let cons = self.insert_node(NodeInner::Constraint(item));
				cons.next.push(dst);
				dst = cons.id;
				self.add_future(dst);
			}
		}

		self.insert_phi_node(dst, live_outs);
	}
}
