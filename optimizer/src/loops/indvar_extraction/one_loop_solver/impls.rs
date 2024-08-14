// 寻找归纳变量的算法
use std::collections::{HashMap, HashSet};

use llvm::{LlvmTemp, LlvmTempManager};
use rrvm::{program::LlvmFunc, rrvm_loop::LoopPtr, LlvmNode};

use crate::{loops::loop_data::LoopData, metadata::FuncData};

use super::{tarjan_var::TarjanVar, OneLoopSolver};

impl<'a> OneLoopSolver<'a> {
	pub fn new(
		func: &'a mut LlvmFunc,
		loopdata: &'a mut LoopData,
		funcdata: &'a mut FuncData,
		temp_mgr: &'a mut LlvmTempManager,
		cur_loop: LoopPtr,
		preheader: LlvmNode,
	) -> Self {
		Self {
			func,
			loopdata,
			funcdata,
			temp_mgr,
			tarjan_var: TarjanVar::new(),
			header_map: HashMap::new(),
			cur_loop,
			preheader,
			useful_variants: HashSet::new(),
			indvars: HashMap::new(),
			new_invariant_instr: HashMap::new(),
			flag: false,
		}
	}
	pub fn classify_indvar(&mut self) {
		let phi_defs: Vec<LlvmTemp> = self
			.cur_loop
			.borrow()
			.header
			.borrow()
			.phi_instrs
			.iter()
			.map(|i| i.target.clone())
			.collect();
		// 先找 header 中的 phi
		for use_ in phi_defs.iter() {
			self.run(use_.clone());
		}
		// 再找其余未被 visit 的变量
		let blocks = self
			.cur_loop
			.borrow()
			.blocks_without_subloops(&self.func.cfg, &self.loopdata.loop_map);
		for block in blocks {
			let block = block.borrow();
			for inst in block.phi_instrs.iter() {
				if !self.tarjan_var.visited.contains(&inst.target) {
					self.run(inst.target.clone());
				}
			}
			for inst in block.instrs.iter() {
				if let Some(t) = inst.get_write() {
					if !self.tarjan_var.visited.contains(&t) {
						self.run(t);
					}
				}
			}
		}
	}
}
