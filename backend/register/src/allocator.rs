use std::collections::HashMap;

use instruction::{riscv::prelude::*, temp::TempManager};
use rrvm::program::RiscvFunc;

use crate::{graph::InterferenceGraph, spill::spill};

#[derive(Default)]
pub struct RegAllocator {}

impl RegAllocator {
	pub fn alloc(&mut self, func: &mut RiscvFunc, mgr: &mut TempManager) {
		let map: HashMap<_, _> = loop {
			func.cfg.analysis();
			let mut graph = InterferenceGraph::new(&func.cfg);
			graph.pre_color();
			graph.merge_nodes();
			if graph.coloring() {
				break graph.color;
			}
			let node = graph.spill_node.unwrap();
			func.spills += 1;
			spill(func, node, (-func.spills * 8, FP.into()).into(), mgr);
		};
		let map = map.into_iter().map(|(k, v)| (k, PhysReg(v))).collect();
		func.cfg.blocks.iter().for_each(|v| v.borrow_mut().map_temp(&map));
	}
}
