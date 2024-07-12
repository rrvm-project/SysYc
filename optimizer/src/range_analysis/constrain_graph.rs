use llvm::{
	ArithInstr, ArithOp, ConvertInstr, ConvertOp, LlvmTemp, PhiInstr, Value,
};
use utils::from_label;

use std::{collections::HashMap, fmt::Debug, vec};

use super::{
	constrain::Constrain,
	range::{Range, RangeItem},
	tarjan::Graph,
};

#[derive(Debug, Clone)]
pub struct ConstrainGraph {
	nodes: Vec<Option<Node>>,
	tmp_to_nodes: HashMap<LlvmTemp, HashMap<i32, usize>>,
}

#[derive(Debug, Clone)]
pub enum NodeInner {
	Temp(LlvmTemp, i32),
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
}

impl Graph<'_> for ConstrainGraph {
	fn next(&'_ self, u: usize) -> Box<dyn Iterator<Item = usize> + '_> {
		Box::new(
			self
				.get_node_ref(u)
				.next()
				.into_iter()
				.cloned()
				.chain(self.get_node_ref(u).future().into_iter().cloned()),
		)
	}
}

impl Node {
	pub fn get_id(&self) -> usize {
		self.id
	}

	pub fn get_inner_range_ref(&self) -> Option<&Range> {
		match &self.inner {
			NodeInner::Constraint(c) => Some(c),
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
		}));

		self.nodes.get_mut(id).unwrap().as_mut().unwrap()
	}

	fn look_up_tmp_node(
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
		fn add_one(this: &mut ConstrainGraph, item: &Option<RangeItem>, id: usize) {
			if let Some((t, block)) = match item {
				Some(RangeItem::IntFuture(t, block, _)) => Some((t, *block)),
				Some(RangeItem::FloatFuture(t, block, _)) => Some((t, *block)),
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
						self.insert_node(NodeInner::Constraint(Range::fromi32(*i)))
					}
					llvm::Value::Float(f) => {
						self.insert_node(NodeInner::Constraint(Range::fromf32(*f)))
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
