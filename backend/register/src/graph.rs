use std::collections::{BinaryHeap, HashMap, HashSet};

use instruction::{riscv::prelude::*, temp::Temp};
use rrvm::RiscvCFG;
use utils::union_find::UnionFind;

#[derive(Default)]
pub struct InterferenceGraph {
	uf: UnionFind<Temp>,
	nodes: HashSet<Temp>,
	colors: HashMap<Temp, RiscvReg>,
	edges: HashMap<Temp, HashSet<Temp>>,
	merge_benefit: HashMap<(Temp, Temp), f64>, // HACK: benefit varies while coalescing ?
}

impl InterferenceGraph {
	fn add_edge(&mut self, x: Temp, y: Temp) {
		if x != y {
			self.edges.entry(x).or_default().insert(y);
			self.edges.entry(y).or_default().insert(x);
		}
	}
	fn can_set(&self, x: &Temp, reg: RiscvReg) -> bool {
		self.edges.get(x).map_or(true, |arr| {
			arr.iter().all(|v| self.colors.get(v).map_or(true, |v| *v != reg))
		})
	}
	fn can_merge(&self, x: &Temp, y: &Temp) -> bool {
		x != y
			&& self.edges.get(x).map_or(true, |s| !s.contains(y))
			&& match (self.colors.get(x), self.colors.get(y)) {
				(Some(x), Some(y)) => x == y,
				(Some(x), None) => self.can_set(y, *x),
				(None, Some(y)) => self.can_set(x, *y),
				_ => true,
			}
	}
	fn get_nodes(&mut self) -> Vec<Temp> {
		self.nodes.iter().copied().filter(|v| self.uf.is_root(*v)).collect()
	}
	fn get_degree(&self, x: &Temp) -> usize {
		self.edges.get(x).map(|v| v.len()).unwrap_or_default()
	}
	fn briggs_cond(&self, x: &Temp, y: &Temp) -> bool {
		self
			.edges
			.get(x)
			.unwrap_or(&HashSet::new())
			.union(self.edges.get(y).unwrap_or(&HashSet::new()))
			.collect::<HashSet<_>>()
			.len() < ALLOACBLE_COUNT
	}
	fn merge(&mut self, x: &Temp, y: &Temp) {
		self.uf.merge(*y, *x);
		if let Some(color) = self.colors.remove(y) {
			self.colors.insert(*x, color);
		}
		if let Some(edges) = self.edges.remove(y) {
			edges.iter().for_each(|v| {
				let entry = self.edges.entry(*v).or_default();
				entry.insert(*x);
				entry.remove(y);
			});
			self.edges.entry(*x).or_default().extend(edges);
		}
	}

	pub fn new(cfg: &RiscvCFG) -> Self {
		let mut graph = Self::default();
		cfg.clear_data_flow();
		cfg.analysis();
		for block in cfg.blocks.iter() {
			let block = &block.borrow();
			let mut lives = block.live_out.clone();
			for instr in block.instrs.iter().rev() {
				if let Some(temp) = instr.get_write() {
					graph.nodes.insert(temp);
					lives.remove(&temp);
					lives.iter().for_each(|x| graph.add_edge(temp, *x));
				}
				for temp in instr.get_read() {
					graph.nodes.insert(temp);
					lives.iter().for_each(|x| graph.add_edge(temp, *x));
					lives.insert(temp);
				}
				if instr.is_move() {
					let x = instr.get_read().pop();
					let y = instr.get_write();
					if let (Some(x), Some(y)) = (x, y) {
						*graph.merge_benefit.entry((x, y)).or_default() += block.weight;
						*graph.merge_benefit.entry((y, x)).or_default() += block.weight;
					}
				}
			}
		}
		graph
	}

	pub fn pre_color(&mut self) {
		for node in self.nodes.iter() {
			if let Some(color) = node.pre_color {
				self.colors.insert(*node, color);
			}
		}
	}

	pub fn eliminate_move(&mut self) {
		let edges = std::mem::take(&mut self.merge_benefit);
		let mut edges = edges.into_iter().collect::<Vec<_>>();
		edges.sort_by(|(_, x), (_, y)| x.total_cmp(y));
		loop {
			let mut flag = true;
			for ((u, v), _) in edges.iter_mut() {
				*u = self.uf.find(*u);
				*v = self.uf.find(*v);
				if self.can_merge(u, v) {
					flag = false;
					self.merge(u, v);
					*v = *u;
				}
			}
			edges.retain(|((x, y), _)| {
				x.id != y.id
					&& match (self.colors.get(x), self.colors.get(y)) {
						(Some(x), Some(y)) => x == y,
						_ => true,
					}
			});
			if flag {
				break;
			}
		}
	}

	pub fn coalescing(&mut self) {
		let mut heap: BinaryHeap<_> =
			self.get_nodes().into_iter().map(|v| (self.get_degree(&v), v)).collect();
		while let Some((d, x)) = heap.pop() {
			if d > ALLOACBLE_COUNT {
				break;
			}
			for (_, y) in heap.iter().take(ALLOACBLE_COUNT * ALLOACBLE_COUNT) {
				if self.can_merge(&x, y) && self.briggs_cond(&x, y) {
					self.merge(y, &x);
					break;
				}
			}
		}
	}

	#[allow(clippy::map_entry)]
	pub fn coloring(&mut self) -> Option<Temp> {
		let mut nodes: Vec<_> =
			self.get_nodes().iter().map(|v| (*v, self.get_degree(v))).collect();
		nodes.sort_by(|(_, x), (_, y)| x.cmp(y));
		for (node, _) in nodes {
			if !self.colors.contains_key(&node) {
				let neighbors = self.edges.remove(&node).unwrap_or_default();
				let used: HashSet<_> =
					neighbors.into_iter().filter_map(|u| self.colors.get(&u)).collect();
				if let Some(&reg) = ALLOCABLE_REGS.iter().find(|v| !used.contains(v)) {
					self.colors.insert(node, reg);
				} else {
					return Some(node);
				}
			}
		}
		None
	}

	pub fn get_map(mut self) -> HashMap<Temp, RiscvReg> {
		self
			.nodes
			.into_iter()
			.map(|v| (v, *self.colors.get(&self.uf.find(v)).unwrap()))
			.collect()
	}
}
