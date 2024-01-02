use std::collections::HashMap;

use instruction::riscv::value::RiscvTemp::PhysReg;
use rrvm::program::RiscvFunc;

use crate::{graph::InterferenceGraph, spill::spill};

#[derive(Default)]
pub struct RegAllocator {}

impl RegAllocator {
	pub fn alloc(&mut self, func: &mut RiscvFunc) {
		let map: HashMap<_, _> = loop {
			func.cfg.analysis();
			let mut graph = InterferenceGraph::new(&func.cfg);
			graph.pre_color();
			graph.merge_nodes();
			if graph.coloring() {
				break graph.color;
			}
			let node = graph.spill_node.unwrap();
			spill(func, node, func.max_temp());
		};
		let map = map.into_iter().map(|(k, v)| (k, PhysReg(v))).collect();
		func.cfg.blocks.iter().for_each(|v| v.borrow_mut().map_temp(&map));
	}
}
