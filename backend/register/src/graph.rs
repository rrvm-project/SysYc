use std::collections::{HashMap, HashSet};

use instruction::{
	riscv::{
		reg::ALLOACBLE_COUNT,
		value::RiscvTemp::{PhysReg, VirtReg},
	},
	temp::Temp,
};
use rrvm::RiscvCFG;

#[derive(Default)]
pub struct InterferenceGraph {
	pub color_cnt: usize,
	pub spill_node: Option<Temp>,
	pub color: HashMap<Temp, usize>,
	pub edge: Vec<(Temp, Temp)>,
	pub merge_benefit: HashMap<(Temp, Temp), f64>,
	pub color_benefit: HashMap<Temp, Vec<f64>>,
	pub spill_cost: HashMap<Temp, f64>,
}

impl InterferenceGraph {
	pub fn new(cfg: &RiscvCFG) -> Self {
		let mut graph = Self::default();
		for node in cfg.blocks.iter() {
			let mut now = node.borrow().live_in.clone();
			for instr in node.borrow_mut().instrs.iter_mut() {
				if let Some(temp) = instr.get_write() {
					if instr.set_start(!now.contains(&temp)) {
						// eprintln!("{instr}");
						now.insert(temp);
					}
				}
			}

			let weight = node.borrow().weight;
			for instr in node.borrow().instrs.iter().rev() {
				// calc graph
				if instr.is_start() {
					instr.get_write().iter().for_each(|v| {
						now.remove(v);
					});
				}
				graph.edge.extend(
					instr
						.get_read()
						.iter()
						.flat_map(|&x| now.iter().flat_map(move |&y| vec![(x, y), (y, x)])),
				);
				now.extend(instr.get_read().iter());
				eprintln!(
					"{}",
					now.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(" ")
				);
				// calc benefit of merge & precolor
				if instr.is_move() {
					let uses = instr.get_riscv_read();
					let defs = instr.get_riscv_write();
					let (mut virt, mut phys) = uses
						.into_iter()
						.chain(defs.into_iter())
						.fold((Vec::new(), Vec::new()), |(mut x, mut y), v| {
							match v {
								_ if v.is_zero() => (),
								PhysReg(reg) => y.push(reg),
								VirtReg(temp) => x.push(temp),
							};
							(x, y)
						});
					assert_eq!(virt.len() + phys.len(), 2);
					if virt.len() == 2 {
						let x = virt.pop().unwrap();
						let y = virt.pop().unwrap();
						if x != y {
							*graph.merge_benefit.entry((x, y)).or_default() += weight;
							*graph.merge_benefit.entry((y, x)).or_default() += weight;
						}
					} else {
						let x = virt.pop().unwrap();
						let y = phys.pop().unwrap();
						graph
							.color_benefit
							.entry(x)
							.or_insert_with(|| vec![0.0; ALLOACBLE_COUNT])[y.get_index()] += weight;
					}
				}
			}
			graph
				.edge
				.extend(now.iter().flat_map(|&x| now.iter().map(move |&y| (x, y))));
		}
		graph.edge =
			graph.edge.into_iter().collect::<HashSet<_>>().into_iter().collect();
		graph.edge.retain(|(x, y)| x != y);
		graph
	}
	pub fn coloring(&mut self) {
		todo!()
	}
}
