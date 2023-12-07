use std::collections::HashMap;

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
		eprintln!("{}", func.cfg);
		let map: HashMap<_, _> = loop {
			func.cfg.analysis();
			let mut graph = InterferenceGraph::new(&func.cfg);
			for (u, v) in graph.edges.iter() {
				eprintln!("{} {}", u.id, v.id);
			}
			if graph.coloring() {
				break graph.color;
			}
			let node = graph.spill_node.unwrap();
			spill(func, node);
		};
		for (k, v) in map.iter() {
			eprintln!("{} {}", k, v);
		}
		func.cfg.blocks.iter().for_each(|v| v.borrow_mut().map_temp(&map));
	}
}
