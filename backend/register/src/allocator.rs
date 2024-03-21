use instruction::{riscv::prelude::*, temp::TempManager};
use rrvm::program::RiscvFunc;

use crate::{graph::InterferenceGraph, spill::spill};

#[derive(Default)]
pub struct RegAllocator {}

impl RegAllocator {
	pub fn alloc(&mut self, func: &mut RiscvFunc, mgr: &mut TempManager) {
		let map = loop {
			func.cfg.analysis();
			let mut graph = InterferenceGraph::new(&func.cfg);
			graph.pre_color();
			graph.eliminate_move();
			graph.coalescing();
			if let Some(node) = graph.coloring() {
				func.spills += 1;
				spill(func, node, (-func.spills * 8, FP.into()).into(), mgr);
			} else {
				break graph.get_map();
			}
		};
		let map = map.into_iter().map(|(k, v)| (k, PhysReg(v))).collect();
		func.cfg.blocks.iter().for_each(|v| v.borrow_mut().map_temp(&map));
	}
}
