use std::{
	collections::{HashMap, HashSet},
	f64::INFINITY,
};

use instruction::{
	riscv::{
		reg::{RiscvReg, ALLOACBLE_COUNT, ALLOCABLE_REGS},
		value::RiscvTemp::VirtReg,
		RiscvInstr,
	},
	temp::Temp,
};
use rrvm::RiscvCFG;
use utils::union_find::UnionFind;

use crate::utils::{get_degree, spill_cost};

#[derive(Default)]
pub struct InterferenceGraph {
	pub temps: Vec<Temp>,
	pub spill_node: Option<Temp>,
	pub union_find: UnionFind<Temp>,
	pub color: HashMap<Temp, RiscvReg>,
	pub edges: Vec<(Temp, Temp)>,
	pub merge_w: HashMap<(Temp, Temp), f64>,
	pub spill_cost: HashMap<Temp, f64>,
}

impl InterferenceGraph {
	pub fn new(cfg: &RiscvCFG) -> Self {
		let mut graph = Self::default();

		macro_rules! edge_extend {
			($a:expr, $b:expr) => {{
				let c = $b; // ???
				graph.edges.extend(
					$a.into_iter()
						.flat_map(|&x| c.iter().flat_map(move |&y| vec![(x, y), (y, x)])),
				);
			}};
		}

		for node in cfg.blocks.iter() {
			let mut now = node.borrow().live_in.clone();
			for instr in node.borrow_mut().instrs.iter_mut() {
				if let Some(temp) = instr.get_write() {
					if instr.set_start(!now.contains(&temp)) {
						now.insert(temp);
					}
				}
			}
			let weight = node.borrow().weight;
			let mut now = node.borrow().live_out.clone();
			let mut last_end = HashSet::new();
			for instr in node.borrow().instrs.iter().rev() {
				// calc graph
				if instr.is_start() {
					instr.get_write().iter().for_each(|v| {
						if !now.remove(v) {
							edge_extend!(Some(v), &now);
						}
					});
				}
				edge_extend!(&instr.get_read(), &now);
				edge_extend!(&instr.get_read(), &instr.get_read());
				// calc spill cost
				if let Some(temp) = instr.get_write() {
					if last_end.contains(&temp) {
						*graph.spill_cost.entry(temp).or_default() = INFINITY;
					}
					graph.temps.push(temp);
					*graph.spill_cost.entry(temp).or_default() += weight;
				}
				let read_set = instr.get_read().into_iter().collect::<HashSet<_>>();
				let diff = read_set.difference(&now).cloned().collect::<HashSet<_>>();
				if !diff.is_empty() {
					last_end = diff;
				}
				for temp in read_set {
					now.insert(temp);
					graph.temps.push(temp);
					*graph.spill_cost.entry(temp).or_default() += weight;
				}
				// calc benefit of merge & precolor
				graph.calc_w(instr, weight);
			}
			edge_extend!(&now, &now);
		}
		graph.edges =
			graph.edges.into_iter().collect::<HashSet<_>>().into_iter().collect();
		graph.edges.retain(|(x, y)| x != y);
		graph.temps =
			graph.temps.into_iter().collect::<HashSet<_>>().into_iter().collect();
		graph
	}

	fn calc_w(&mut self, instr: &RiscvInstr, weight: f64) {
		if instr.is_move() {
			let uses = instr.get_riscv_read();
			let defs = instr.get_riscv_write();
			let mut virt =
				uses.into_iter().chain(defs).fold(Vec::new(), |mut x, v| {
					if let VirtReg(temp) = v {
						x.push(temp);
					}
					x
				});
			if virt.len() == 2 {
				let x = virt.pop().unwrap();
				let y = virt.pop().unwrap();
				if x != y {
					*self.merge_w.entry((x, y)).or_default() += weight;
					*self.merge_w.entry((y, x)).or_default() += weight;
				}
			}
		}
	}

	pub fn pre_color(&mut self) {
		for temp in self.temps.iter() {
			if let Some(reg) = temp.pre_color {
				eprintln!("pre color {temp} {reg}");
				self.color.insert(*temp, reg);
			}
		}
	}

	pub fn merge_nodes(&mut self) {
		let mut edges: HashMap<Temp, Vec<Temp>> = HashMap::new();
		for (u, v) in self.edges.iter() {
			edges.entry(*u).or_default().push(*v);
		}
		let mut to_merge: Vec<_> = self
			.merge_w
			.iter()
			.filter(|((x, y), _)| x < y)
			.map(|((u, v), w)| (u, v, w))
			.collect();
		to_merge.sort_by(|(_, _, x), (_, _, y)| y.total_cmp(x));

		loop {
			let mut flag = true;
			for (&x, &y, _) in to_merge.iter() {
				if !self.union_find.same(x, y) {
					let x = self.union_find.find(x);
					let y = self.union_find.find(y);
					let color_conflict = match (self.color.get(&x), self.color.get(&y)) {
						(Some(reg_x), Some(reg_y)) => reg_x != reg_y,
						_ => false,
					};
					let not_adjust =
						edges.get(&x).map_or(true, |e| e.iter().all(|&v| v != y));
					if not_adjust && !color_conflict {
						// x 和 y 的邻居节点中 >= N 的 小于 N ？
						let a = edges.get(&x).cloned().unwrap_or_else(Vec::new);
						let b = edges.get(&y).cloned().unwrap_or_else(Vec::new);
						let neighbors: HashSet<_> = a
							.into_iter()
							.chain(b.into_iter())
							.filter(|v| {
								get_degree(v, &edges, &mut self.union_find) >= ALLOACBLE_COUNT
							})
							.collect();
						if neighbors.len() < ALLOACBLE_COUNT {
							let a = edges.get(&x).cloned().unwrap_or_else(Vec::new);
							a.iter().for_each(|v| edges.entry(*v).or_default().push(y));
							edges.entry(y).or_default().extend(a);
							self.union_find.merge(x, y);
							if let Some(reg_x) = self.color.get(&x) {
								self.color.insert(y, *reg_x);
							}
							flag = false;
						}
					}
				}
			}
			if flag {
				break;
			}
		}
		self.edges = edges
			.into_iter()
			.flat_map(|(x, y)| y.into_iter().map(|v| (x, v)).collect::<Vec<_>>())
			.filter(|(x, y)| {
				self.union_find.is_root(*x) && self.union_find.is_root(*y)
			})
			.collect();
	}

	pub fn coloring(&mut self) -> bool {
		let mut edges = HashMap::new();
		for (u, v) in self.edges.iter() {
			edges.entry(u).or_insert_with(Vec::new).push(v);
		}
		let mut temps = self
			.temps
			.iter()
			.filter(|v| !self.color.contains_key(v))
			.map(|v| {
				let degree = edges.get(v).map(|arr| arr.len()).unwrap_or_default();
				let weight = self.spill_cost.get(v).copied().unwrap_or(0.0);
				(spill_cost(weight, degree), v)
			})
			.collect::<Vec<_>>();

		temps.sort_by(|(x, _), (y, _)| y.total_cmp(x));
		for (_, temp) in temps.iter() {
			if self.union_find.is_root(**temp) {
				let used: HashSet<_> = edges
					.remove(temp)
					.unwrap_or_else(Vec::new)
					.iter()
					.filter_map(|v| self.color.get(*v))
					.collect();
				if let Some(reg) = ALLOCABLE_REGS.iter().find(|v| !used.contains(v)) {
					self.color.insert(**temp, *reg);
				} else {
					self.spill_node = Some(**temp);
					return false;
				}
			}
		}
		for (_, &temp) in temps {
			if !self.union_find.is_root(temp) {
				let v = self.color.get(&self.union_find.find(temp)).unwrap();
				self.color.insert(temp, *v);
			}
		}
		for (temp, reg) in self.color.iter() {
			eprintln!("{temp} {reg}")
		}
		true
	}
}
