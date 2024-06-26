use std::collections::HashMap;

use instruction::{
	riscv::{prelude::*, virt_mem::VirtMemManager},
	temp::TempManager,
	Temp,
};
use rrvm::program::RiscvFunc;

use crate::{graph::InterferenceGraph, spill::spill};

pub struct RegAllocator<'a> {
	mgr: &'a mut TempManager,
	mem_mgr: &'a mut VirtMemManager,
}

impl<'a> RegAllocator<'a> {
	pub fn new(
		mgr: &'a mut TempManager,
		mem_mgr: &'a mut VirtMemManager,
	) -> Self {
		Self { mgr, mem_mgr }
	}

	pub fn alloc(
		&mut self,
		func: &mut RiscvFunc,
		mapper: &mut HashMap<Temp, RiscvTemp>,
	) {
		let map = loop {
			let mut graph = InterferenceGraph::new(Box::new(ALLOCABLE_REGS));

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
							graph.add_weight($temp, block.weight);
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
				spill(func, &nodes, self.mgr, self.mem_mgr);
				for block in func.cfg.blocks.iter() {
					let block = &mut block.borrow_mut();
					block.live_out.retain(|v| !nodes.contains(v));
				}
			}
		};
		let map = map.into_iter().map(|(k, v)| (k, PhysReg(v))).collect();
		for block in func.cfg.blocks.iter() {
			let block = &mut block.borrow_mut();
			block.instrs.iter_mut().for_each(|v| v.map_temp(&map))
		}
		mapper.extend(map);
	}
}
