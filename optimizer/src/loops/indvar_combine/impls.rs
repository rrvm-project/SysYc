use std::collections::{HashMap, HashSet};

use llvm::{
	GEPInstr, LlvmInstr, LlvmTemp, LlvmTempManager, Value,
	VarType,
};
use rrvm::{dominator::LlvmDomTree, program::LlvmFunc, rrvm_loop::LoopPtr};

use crate::{
	loops::{
		indvar::IndVar, indvar_type::IndVarType, loop_data::LoopData,
	},
	metadata::FuncData,
};

use super::IndvarCombine;

impl<'a> IndvarCombine<'a> {
	pub fn new(
		func: &'a mut LlvmFunc,
		loopdata: &'a mut LoopData,
		funcdata: &'a mut FuncData,
		temp_mgr: &'a mut LlvmTempManager,
	) -> Self {
		let dom_tree = LlvmDomTree::new(&func.cfg, false);
		Self {
			func,
			loopdata,
			funcdata,
			temp_mgr,
			dom_tree,
		}
	}
	// 按 dfs 序逐个 loop 处理
	pub fn apply(mut self) -> bool {
		self.loopdata.rebuild(self.func);
		let mut flag = false;
		let mut dfs_vec = Vec::new();
		fn dfs(node: LoopPtr, dfs_vec: &mut Vec<LoopPtr>) {
			for subloop in node.borrow().subloops.iter() {
				dfs(subloop.clone(), dfs_vec);
			}
			dfs_vec.push(node);
		}
		dfs(self.loopdata.root_loop.clone(), &mut dfs_vec);
		// 移去 root_node
		dfs_vec.pop();
		for loop_node in dfs_vec.iter() {
			let indvars = self
				.loopdata
				.indvars
				.clone()
				.into_iter()
				.filter(|(t, _)| self.def_loop(t).borrow().id == loop_node.borrow().id)
				.collect();
			flag |= self.indvar_combine(loop_node.clone(), indvars);
		}
		flag
	}
	// 1. 找等价类
	// 2. 为每个等价类找锚点
	pub fn indvar_combine(
		&mut self,
		loop_: LoopPtr,
		indvars: HashMap<LlvmTemp, IndVar>,
	) -> bool {
		let mut combine_map: HashMap<LlvmTemp, Vec<(LlvmTemp, i32)>> =
			HashMap::new();
		let mut pivot_coverage: HashMap<LlvmTemp, Vec<(LlvmTemp, i32)>> =
			HashMap::new();
		for (temp, iv) in indvars.iter() {
			if temp.var_type == VarType::I32 {
				continue;
			}
			// eprintln!("Indvar: {:?}, {}", temp, iv);
			let mut found = false;
			for (k, v) in combine_map.iter_mut() {
				let iv_k = indvars[k].clone();
				// iv = iv_k + dist
				if let Some(dist) = iv_k.has_constant_distance(iv, self.funcdata) {
					v.push((temp.clone(), dist));
					found = true;
					break;
				}
			}
			if !found {
				let mut set = Vec::new();
				set.push((temp.clone(), 0));
				combine_map.insert(temp.clone(), set);
			}
		}
		for (_, v) in combine_map.iter_mut() {
			// eprintln!("{}", k);
			// v 按照距离排序
			v.sort_by(|a, b| a.1.cmp(&b.1));
			// v.iter().for_each(|(k, dist)| eprint!("{}, {} ", k, dist));
			// eprintln!();
			let mut last_uncovered_point_outer = v.first().cloned();
			let mut v_iter = v.iter();
			while let Some(last_uncovered_point) = last_uncovered_point_outer.clone()
			{
				let mut pivot_iter = v_iter.clone();
				let mut pivot = last_uncovered_point.clone();
				// [-2048, 2047]
				while let Some(next_v) = pivot_iter.next() {
					if self.loopdata.scc_map.contains_key(&next_v.0)
						&& !self.loopdata.temp_graph.temp_to_instr[&next_v.0].instr.is_phi()
					{
						continue;
					}
					if next_v.1 - last_uncovered_point.1 < 2048 {
						pivot = next_v.clone();
					} else {
						break;
					}
				}
				// eprintln!("found pivot: {}", pivot.0);
				let pivot_scc =
					if self.loopdata.temp_graph.temp_to_instr[&pivot.0].instr.is_phi() {
						self.loopdata.scc_map[&pivot.0].clone()
					} else {
						vec![pivot.0.clone()]
					};
				// pivot_scc.iter().for_each(|t| eprintln!("scc: {}", t));
				while let Some(mut pivot_covered) = v_iter.next().cloned() {
					if pivot_scc.contains(&pivot_covered.0) {
						continue;
					}
					if pivot_covered.1 - pivot.1 < 2047 {
						pivot_covered.1 = pivot_covered.1 - pivot.1;
						pivot_coverage
							.entry(pivot.0.clone())
							.or_insert(vec![])
							.push(pivot_covered.clone());
					} else {
						last_uncovered_point_outer = Some(pivot_covered);
						break;
					}
				}
				if v_iter.next().is_none() {
					break;
				}
			}
		}
		// 把每个 pivot 做成环，放到 header 中，把它所控制的变量的定义换掉，其中有些变量是 phi
		for (pivot, covers) in pivot_coverage {
			if self.try_strength_reduce(&pivot, &indvars[&pivot], loop_.clone()) {
				let mut new_instrs = HashMap::new();
				let mut phi_cover = HashSet::new();
				let mut phi_cover_instr: Vec<LlvmInstr> = Vec::new();
				for cover in covers {
					eprintln!("pivot {} cover {} {}", pivot, cover.0, cover.1);
					if cover.0.var_type == VarType::I32Ptr
						|| cover.0.var_type == VarType::F32Ptr
					{
						let new_instr = GEPInstr {
							target: cover.0.clone(),
							var_type: cover.0.var_type,
							addr: pivot.clone().into(),
							offset: Value::Int(cover.1.clone()),
						};
						if self.loopdata.temp_graph.temp_to_instr[&cover.0].instr.is_phi() {
							phi_cover.insert(cover.0.clone());
							phi_cover_instr.push(Box::new(new_instr));
						} else {
							new_instrs.insert(cover.0.clone(), Box::new(new_instr));
						}
					}
				}
				loop_
					.borrow_mut()
					.header
					.borrow_mut()
					.phi_instrs
					.retain(|phi| !phi_cover.contains(&phi.target));
				phi_cover_instr
					.append(&mut loop_.borrow_mut().header.borrow_mut().instrs);
				loop_.borrow_mut().header.borrow_mut().instrs = phi_cover_instr;
				for block in loop_
					.borrow()
					.blocks_without_subloops(&self.func.cfg, &self.loopdata.loop_map)
				{
					for instr in block.borrow_mut().instrs.iter_mut() {
						if let Some(w) = instr.get_write() {
							if let Some(new_instr) = new_instrs.get(&w) {
								*instr = new_instr.clone();
							}
						}
					}
				}
			}
		}
		false
	}
	// 某变量定义在哪个循环中
	pub fn def_loop(&self, temp: &LlvmTemp) -> LoopPtr {
		if let Some(bb) = self.loopdata.def_map.get(temp) {
			self.loopdata.loop_map[&bb.borrow().id].clone()
		} else {
			// 找不到定义的 temp 就被视为定义在 root_loop 中
			self.loopdata.root_loop.clone()
		}
	}
	pub fn try_strength_reduce(
		&mut self,
		target: &LlvmTemp,
		iv: &IndVar,
		loop_: LoopPtr,
	) -> bool {
		// 被我 reduce 的 pivot 所在的基本块一定要支配 loop 所有的 latch 块
		let def_bb = self.loopdata.def_map[target].clone();
		let latch_bbs = loop_.borrow().get_loop_latches(&self.loopdata.loop_map);
		if latch_bbs.iter().any(|latch_bb| {
			!self.dom_tree.dominates[&def_bb.borrow().id].contains(latch_bb)
		}) {
			return false;
		} else {
			if iv.get_type() == IndVarType::Ordinary {
				if self.loopdata.temp_graph.temp_to_instr[target].instr.is_phi() {
					return true;
				} else {
					return false;
				}
			} else {
				#[cfg(feature = "debug")]
				eprintln!("SR: not reducing iv: {}", iv);
				false
			}
		}
	}
}
