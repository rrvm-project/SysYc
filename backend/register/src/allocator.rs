use instruction::{riscv::prelude::*, temp::TempManager};
use rrvm::program::RiscvFunc;

use crate::{graph::InterferenceGraph, spill::spill};

#[derive(Default)]
pub struct RegAllocator {}

impl RegAllocator {
	pub fn alloc(&mut self, func: &mut RiscvFunc, mgr: &mut TempManager) {
		let map = loop {
			let mut graph = InterferenceGraph::new(Box::new(ALLOCABLE_REGS));
			func.cfg.clear_data_flow();
			func.cfg.analysis();

			for block in func.cfg.blocks.iter() {
				let block = &block.borrow();
				let mut lives = block.live_out.clone();
				for instr in block.instrs.iter().rev() {
					macro_rules! add_node {
						($temp:expr) => {
							if let Some(col) = $temp.pre_color {
								graph.set_color(&$temp, col);
							}
							lives.iter().for_each(|x| graph.add_edge($temp, *x));
							graph.add_node($temp);
						};
					}
					if let Some(temp) = instr.get_write() {
						lives.remove(&temp);
						add_node!(temp);
					}
					for temp in instr.get_read() {
						add_node!(temp);
						lives.insert(temp);
					}
					if instr.is_move() {
						let x = instr.get_read().pop();
						let y = instr.get_write();
						if let (Some(x), Some(y)) = (x, y) {
							graph.add_benefit(&x, &y, block.weight);
						}
					}
				}
			}

			graph.eliminate_move();
			graph.coalescing();
			let nodes = graph.coloring();
			if nodes.is_empty() {
				break graph.get_map();
			} else {
				for node in nodes {
					func.spills += 1;
					spill(func, node, (-func.spills * 8, FP.into()).into(), mgr);
				}
			}
		};
		let map = map.into_iter().map(|(k, v)| (k, PhysReg(v))).collect();
		func.cfg.blocks.iter().for_each(|v| v.borrow_mut().map_temp(&map));
	}
}
