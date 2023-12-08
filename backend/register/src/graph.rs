use std::{
	cmp::Ordering,
	collections::{HashMap, HashSet},
};

use instruction::{
	riscv::{
		reg::{RiscvReg, ALLOACBLE_COUNT, ALLOCABLE_REGS},
		value::RiscvTemp::{PhysReg, VirtReg},
		RiscvInstr,
	},
	temp::Temp,
};
use rrvm::RiscvCFG;

#[derive(Default)]
pub struct InterferenceGraph {
	pub temps: Vec<Temp>,
	pub spill_node: Option<Temp>,
	pub color: HashMap<Temp, RiscvReg>,
	pub edges: Vec<(Temp, Temp)>,
	pub merge_w: HashMap<(Temp, Temp), f64>,
	pub color_w: HashMap<Temp, Vec<f64>>,
	pub spill_cost: HashMap<Temp, f64>,
}

fn default_array() -> Vec<f64> {
	(0..ALLOACBLE_COUNT).map(|v| 0.1 * v as f64).collect()
}

fn cmp_tuple<T>((_, x): &(T, f64), (_, y): &(T, f64)) -> Ordering {
	x.total_cmp(y)
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
				now.extend(instr.get_read().iter());
				if let Some(temp) = instr.get_write() {
					graph.temps.push(temp);
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
			let (mut virt, mut phys) = uses.into_iter().chain(defs).fold(
				(Vec::new(), Vec::new()),
				|(mut x, mut y), v| {
					match v {
						_ if v.is_zero() => (),
						PhysReg(reg) => y.push(reg),
						VirtReg(temp) => x.push(temp),
					};
					(x, y)
				},
			);
			assert_eq!(virt.len() + phys.len(), 2);
			if virt.len() == 2 {
				let x = virt.pop().unwrap();
				let y = virt.pop().unwrap();
				if x != y {
					*self.merge_w.entry((x, y)).or_default() += weight;
					*self.merge_w.entry((y, x)).or_default() += weight;
				}
			} else {
				let x = virt.pop().unwrap();
				let y = phys.pop().unwrap();
				self.color_w.entry(x).or_insert_with(default_array)[y.get_index()] +=
					weight;
			}
		}
	}

	pub fn coloring(&mut self) -> bool {
		let mut edges = HashMap::new();
		let mut color = HashMap::new();
		for (u, v) in self.edges.iter() {
			edges.entry(u).or_insert_with(Vec::new).push(v);
		}
		let mut temps = self
			.temps
			.iter()
			.map(|v| (edges.get(v).map(|arr| arr.len()).unwrap_or_default(), v))
			.collect::<Vec<_>>();
		temps.sort_by_key(|v| v.0);
		for (_, temp) in temps {
			let mut a: Vec<_> = self
				.color_w
				.remove(temp)
				.unwrap_or_else(default_array)
				.into_iter()
				.enumerate()
				.collect();
			a.sort_by(cmp_tuple);
			let used: HashSet<_> = edges
				.remove(temp)
				.unwrap_or_else(Vec::new)
				.iter()
				.filter_map(|v| color.get(v))
				.collect();
			if let Some(reg) = a.into_iter().find(|(index, _)| !used.contains(index))
			{
				color.insert(temp, reg.0);
			} else {
				return false;
			}
		}
		self.color = color
			.into_iter()
			.map(|(k, v)| (*k, *ALLOCABLE_REGS.get(v).unwrap()))
			.collect();
		true
	}
}
