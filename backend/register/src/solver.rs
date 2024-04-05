use std::collections::{HashMap, HashSet};

use instruction::{
	riscv::{prelude::*, virt_mem::VirtMemManager},
	temp::TempManager,
	Temp,
};
use rrvm::program::RiscvFunc;
use utils::math::align16;

use crate::{
	allocator::RegAllocator, graph::InterferenceGraph, utils::MemAllocator,
};

pub struct RegisterSolver<'a> {
	mgr: &'a mut TempManager,
	mem_mgr: VirtMemManager,
	temp_mapper: HashMap<Temp, RiscvTemp>,
}

impl<'a> RegisterSolver<'a> {
	pub fn new(mgr: &'a mut TempManager) -> Self {
		Self {
			mgr,
			mem_mgr: VirtMemManager::default(),
			temp_mapper: HashMap::new(),
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

		RegAllocator::new(self.mgr, &mut self.mem_mgr)
			.alloc(func, &mut self.temp_mapper);
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

	pub fn solve_caller_save(&mut self, func: &mut RiscvFunc) {
		for block in func.cfg.blocks.iter() {
			let block = &mut block.borrow_mut();
			let mut lives: HashSet<_> = block
				.live_out
				.iter()
				.filter_map(|v| self.temp_mapper.get(v))
				.copied()
				.collect();
			let mut to_save = None;
			let mut instrs = std::mem::take(&mut block.instrs);
			for instr in instrs.iter_mut().rev() {
				for temp in instr.get_riscv_write() {
					lives.remove(&temp);
				}
				for temp in instr.get_riscv_read() {
					lives.insert(temp);
				}
				match instr.get_temp_op() {
					Some(Save) => instr.set_lives(to_save.take().unwrap()),
					Some(Restore) => {
						let lives: Vec<_> = lives
							.iter()
							.filter_map(|v| v.get_phys())
							.filter(|v| CALLER_SAVE.iter().skip(1).any(|reg| reg == v))
							.collect();
						instr.set_lives(lives.clone());
						to_save = Some(lives);
					}
					_ => (),
				}
			}

			block.instrs = instrs
				.into_iter()
				.flat_map(|instr| match instr.get_temp_op() {
					Some(Save) => {
						let lives = instr.get_lives();
						let size = align16(lives.len() as i32 * 8);
						let prelude =
							ITriInstr::new(Addi, SP.into(), SP.into(), (-size).into());
						let mut instrs = vec![prelude];
						for (index, v) in lives.into_iter().enumerate() {
							let p = (index * 8) as i32;
							instrs.push(IBinInstr::new(SD, v.into(), (p, SP.into()).into()));
						}
						instrs
					}
					Some(Restore) => {
						let lives = instr.get_lives();
						let size = align16(lives.len() as i32 * 8);
						let mut instrs = Vec::new();
						for (index, v) in lives.into_iter().enumerate() {
							let p = (index * 8) as i32;
							instrs.push(IBinInstr::new(LD, v.into(), (p, SP.into()).into()));
						}
						let epilogue =
							ITriInstr::new(Addi, SP.into(), SP.into(), size.into());
						instrs.push(epilogue);
						instrs
					}
					None => vec![instr],
				})
				.collect()
		}
	}
}
