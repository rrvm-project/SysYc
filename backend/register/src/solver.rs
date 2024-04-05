use std::collections::HashSet;

use instruction::{
	riscv::{prelude::*, virt_mem::VirtMemManager},
	temp::TempManager,
};
use rrvm::program::RiscvFunc;

use crate::{
	allocator::RegAllocator, graph::InterferenceGraph, utils::MemAllocator,
};

pub struct RegisterSolver<'a> {
	mgr: &'a mut TempManager,
	mem_mgr: VirtMemManager,
}

impl<'a> RegisterSolver<'a> {
	pub fn new(mgr: &'a mut TempManager) -> Self {
		Self {
			mgr,
			mem_mgr: VirtMemManager::default(),
		}
	}

	pub fn solve_parameter(&mut self, func: &mut RiscvFunc) {
		let mut prelude = Vec::new();
		for (temp, reg) in func.params.iter().zip(PARAMETER_REGS.iter()).rev() {
			let reg = self.mgr.new_pre_color_temp(*reg);
			let temp = self.mgr.get(&temp.into());
			let instr = RTriInstr::new(Add, temp, reg, X0.into());
			prelude.push(instr);
		}
		for (index, param) in func.params.iter().skip(8).enumerate() {
			let temp = self.mgr.get(param.unwrap_temp().as_ref().unwrap());
			let addr = self.mem_mgr.new_mem_with_addr(index as i32);
			self.mem_mgr.set_addr(temp, addr);
			let instr = IBinInstr::new(LD, temp, addr.into());
			prelude.push(instr);
		}
		func.cfg.blocks.first().unwrap().borrow_mut().instrs.splice(0..0, prelude);
	}

	#[allow(clippy::assigning_clones)]
	pub fn register_alloc(&mut self, func: &mut RiscvFunc) {
		func.cfg.clear_data_flow();
		func.cfg.analysis();

		// Original live-out set is needed for virtual memory liveness analysis
		for block in func.cfg.blocks.iter() {
			let block = &mut block.borrow_mut();
			block.live_in = block.live_out.clone();
		}

		RegAllocator::new(self.mgr, &mut self.mem_mgr).alloc(func);
	}

	pub fn memory_alloc(&mut self, func: &mut RiscvFunc) {
		let mut graph = InterferenceGraph::new(Box::<MemAllocator>::default());

		for block in func.cfg.blocks.iter() {
			let block = &block.borrow();
			let mut lives: HashSet<_> = block
				.live_in
				.difference(&block.live_out)
				.map(|v| self.mem_mgr.get_mem((*v).into()))
				.collect();

			for instr in block.instrs.iter().rev() {
				macro_rules! add_node {
					($addr:expr) => {
						if let Some(col) = $addr.pre_color {
							graph.set_color(&$addr, col);
						}
						lives.iter().for_each(|x| graph.add_edge($addr, *x));
						graph.add_weight($addr, 1f64);
					};
				}
				if let Some(addr) = instr.get_virt_mem_write() {
					lives.remove(&addr);
					add_node!(addr);
				}
				if let Some(addr) = instr.get_virt_mem_read() {
					add_node!(addr);
					lives.insert(addr);
				}
			}
		}

		assert!(graph.coloring().is_empty());
		func.spills = graph.get_colors() as i32;
		let map = graph
			.get_map()
			.into_iter()
			.map(|(k, v)| (k, (v * 8, FP.into())))
			.collect();
		for block in func.cfg.blocks.iter() {
			let block = &mut block.borrow_mut();
			block.instrs.iter_mut().for_each(|v| v.map_virt_mem(&map))
		}
	}
}
