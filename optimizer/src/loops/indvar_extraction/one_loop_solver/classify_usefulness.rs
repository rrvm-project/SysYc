use std::collections::{HashMap, HashSet, VecDeque};

use llvm::{LlvmInstrVariant, LlvmTemp};
use utils::{UseTemp, MAX_PHI_NUM};

use super::OneLoopSolver;

impl<'a> OneLoopSolver<'a> {
	// 确定哪些 indvar 是有用的，不能外推的
	// 本身就有用的语句：store, call, 非 indvar header 的 phi, jumpcond 的 use
	// 以及被子循环 use 的变量
	// 如何找被子循环 use 的变量？ 让子循环层层上报自己用了哪些循环外的变量
	pub fn classify_usefulness(&mut self, phi_num: usize) {
		let mut work = VecDeque::new();
		let blocks = self
			.cur_loop
			.borrow()
			.blocks_without_subloops(&self.func.cfg, &self.loopdata.loop_map);
		let used_in_subloop: HashSet<LlvmTemp> = self
			.outside_use
			.iter()
			.filter(|t| self.is_temp_in_current_loop(t))
			.cloned()
			.collect();
		self.outside_use.retain(|t| !used_in_subloop.contains(t));
		#[cfg(feature = "debug")]
		eprintln!("used in subloop: {:?}", used_in_subloop);
		work.extend(used_in_subloop.iter().map(|t| self.header_of_temp(t)));
		let mut reduce_map = HashMap::new();
		for block in blocks.iter() {
			for instr in block.borrow().phi_instrs.iter() {
				if !self.indvars.contains_key(&instr.target) {
					work.push_back(self.header_of_temp(&instr.target));
				}
				for read in instr.get_read() {
					if !self.is_temp_in_current_loop(&read) {
						self.outside_use.insert(read.clone());
					}
				}
			}
			for instr in block.borrow().instrs.iter() {
				match instr.get_variant() {
					LlvmInstrVariant::StoreInstr(inst) => {
						let reads = self.find_temp_to_reduce(
							inst.get_read(),
							&mut reduce_map,
							phi_num,
						);
						work.extend(reads.iter().map(|t| self.header_of_temp(t)));
					}
					LlvmInstrVariant::CallInstr(inst) => {
						work.push_back(self.header_of_temp(&inst.target));
					}
					_ => {}
				}
				for read in instr.get_read() {
					if !self.is_temp_in_current_loop(&read) {
						self.outside_use.insert(read.clone());
					}
				}
			}
			for instr in block.borrow().jump_instr.iter() {
				let reads =
					self.find_temp_to_reduce(instr.get_read(), &mut reduce_map, phi_num);
				work.extend(reads.iter().map(|t| self.header_of_temp(t)));
				for read in instr.get_read() {
					if !self.is_temp_in_current_loop(&read) {
						self.outside_use.insert(read.clone());
					}
				}
			}
		}
		while let Some(temp) = work.pop_front() {
			self.useful_variants.extend(self.scc_of_temp(&temp));
			let mut reads = self
				.reads_of_temp_in_scc(&temp)
				.into_iter()
				// 过滤掉不在当前循环中的变量
				.filter(|t| self.def_loop(t).borrow().id == self.cur_loop.borrow().id)
				// 按 scc 缩点
				.map(|t| self.header_map.get(&t).cloned().unwrap_or(t))
				// 过滤掉已经被标记为 useful 的变量
				.filter(|t| !self.useful_variants.contains(t))
				.collect::<Vec<LlvmTemp>>();
			reads = self.find_temp_to_reduce(reads, &mut reduce_map, phi_num);
			work.extend(reads);
		}
		let mut block_in_cur_loop = self
			.cur_loop
			.borrow()
			.blocks_without_subloops(&self.func.cfg, &self.loopdata.loop_map);
		for block in block_in_cur_loop.iter_mut() {
			for instr in block.borrow_mut().instrs.iter_mut() {
				if let Some(t) = instr.get_write() {
					if let Some(v) = reduce_map.get(&t) {
						let new_instr = self
							.loopdata
							.temp_graph
							.temp_to_instr
							.get(v)
							.unwrap()
							.instr
							.clone();
						*instr = new_instr;
					}
				}
			}
		}
		#[cfg(feature = "debug")]
		eprint!("classified useful variants: ");
		#[cfg(feature = "debug")]
		self.useful_variants.iter().for_each(|t| {
			eprint!("{} ", t);
		});
		#[cfg(feature = "debug")]
		eprintln!();
	}
	fn find_temp_to_reduce(
		&mut self,
		mut reads: Vec<LlvmTemp>,
		reduce_map: &mut HashMap<LlvmTemp, LlvmTemp>,
		mut phi_num: usize,
	) -> Vec<LlvmTemp> {
		let mut to_remove = HashSet::new();
		for read in reads.iter() {
			if let Some(iv) = self.indvars.get(read).cloned() {
				if !self.header_map.contains_key(read) && !reduce_map.contains_key(read)
				{
					if phi_num >= MAX_PHI_NUM {
						break;
					}
					phi_num += 1;
					let flag = self.try_strength_reduce(read, &iv, reduce_map);
					if flag {
						to_remove.insert(read.clone());
					}
				}
			}
		}
		reads.retain(|t| !to_remove.contains(t));
		reads
	}
}
