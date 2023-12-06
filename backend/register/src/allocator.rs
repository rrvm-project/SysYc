use std::collections::HashMap;

use instruction::riscv::reg::{ALLOACBLE_COUNT, ALLOCABLE_REGS};
use rrvm::program::RiscvFunc;

use crate::{graph::InterferenceGraph, spill::spill};

pub struct RegAllocator {}

impl Default for RegAllocator {
	fn default() -> Self {
		Self::new()
	}
}

impl RegAllocator {
	pub fn new() -> Self {
		Self {}
	}
	pub fn alloc(&mut self, func: &mut RiscvFunc) {
		let map: HashMap<_, _> = loop {
			func.cfg.analysis();
			let mut graph = InterferenceGraph::new(&func.cfg);
			graph.coloring();
			if graph.color_cnt <= ALLOACBLE_COUNT {
				break graph
					.color
					.into_iter()
					.map(|(k, v)| (k, *ALLOCABLE_REGS.get(v).unwrap()))
					.collect();
			}
			let node = graph.spill_node.unwrap();
			spill(func, node);
		};
		func.cfg.blocks.iter().for_each(|v| v.borrow_mut().map_temp(&map));
	}
}
