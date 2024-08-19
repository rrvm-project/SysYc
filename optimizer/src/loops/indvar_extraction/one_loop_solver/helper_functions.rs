use std::collections::VecDeque;

use crate::loops::{indvar::IndVar, loopinfo::LoopInfo};

use super::OneLoopSolver;

use llvm::{compute_two_value, CompOp, LlvmInstr, LlvmTemp, Value};
use rrvm::{rrvm_loop::LoopPtr, LlvmNode};
impl<'a> OneLoopSolver<'a> {
	pub fn stack_push(&mut self, temp: LlvmTemp) {
		self.tarjan_var.stack.push(temp.clone());
		self.tarjan_var.in_stack.insert(temp);
	}
	pub fn stack_pop(&mut self) -> Option<LlvmTemp> {
		let temp = self.tarjan_var.stack.pop();
		if let Some(t) = &temp {
			self.tarjan_var.in_stack.remove(t);
		}
		temp
	}
	pub fn stack_contains(&self, temp: &LlvmTemp) -> bool {
		self.tarjan_var.in_stack.contains(temp)
	}
	// 找到 scc 中在 header 中的 phi 语句
	pub fn find_header_for_scc(&mut self, scc: &Vec<LlvmTemp>) -> LlvmTemp {
		for temp in scc {
			if self.loopdata.temp_graph.is_phi(temp)
				&& self.loopdata.def_map[temp].borrow().id
					== self.cur_loop.borrow().header.borrow().id
			{
				return temp.clone();
			}
		}
		unreachable!()
	}
	pub fn get_variant_and_step(
		&mut self,
		member1: &Value,
		member2: &Value,
		header: &LlvmTemp,
	) -> Option<(LlvmTemp, Vec<Value>)> {
		let mut get_variant_and_step_inner =
			|m1: &Value, m2: &Value| -> Option<(LlvmTemp, Vec<Value>)> {
				if let Some(t) = m1.unwrap_temp() {
					if self.header_map.get(&t).is_some_and(|t| t == header) {
						if self.is_loop_invariant(m2) {
							return Some((t, vec![m2.clone()]));
						} else {
							let m2_temp = m2.unwrap_temp().unwrap();
							if !self.tarjan_var.visited.contains(&m2_temp) {
								// 还没有被 tarjan 找过 scc 的话，现在找
								self.tarjan(m2_temp.clone());
							}
							if let Some(iv) = self.indvars.get(&m2_temp) {
								let mut step = vec![iv.base.clone()];
								step.extend(iv.step.clone());
								return Some((t, step));
							}
						}
					}
				}
				None
			};

		get_variant_and_step_inner(member1, member2)
			.or_else(|| get_variant_and_step_inner(member2, member1))
	}
	pub fn is_temp_in_current_loop(&self, temp: &LlvmTemp) -> bool {
		self.def_loop(temp).borrow().id == self.cur_loop.borrow().id
	}
	pub fn is_indvar(&self, value: &Value) -> Option<IndVar> {
		self
			.is_loop_invariant(value)
			.then(|| IndVar::from_loop_invariant(value.clone()))
			.or(value.unwrap_temp().and_then(|temp| self.indvars.get(&temp).cloned()))
	}
	// 只要定义所在的循环不是本循环的子循环即可
	pub fn is_loop_invariant(&self, value: &Value) -> bool {
		match value {
			Value::Temp(temp) => self.loopdata.def_map.get(temp).map_or(true, |bb| {
				!self
					.cur_loop
					.borrow()
					.is_super_loop_of(&self.loopdata.loop_map[&bb.borrow().id])
			}),
			Value::Int(_) => true,
			Value::Float(_) => true,
		}
	}
	pub fn compute_loop_cnt(&mut self, info: &LoopInfo) -> Value {
		let start = &info.begin;
		let step = &info.step;
		let end = &info.end;
		let op = info.comp_op;
		match op {
			CompOp::SLT | CompOp::SGT => {
				// (end - start + step - 1) / step;
				let (tmp1, instr) = compute_two_value(
					end.clone(),
					start.clone(),
					llvm::ArithOp::Sub,
					self.temp_mgr,
				);
				instr.map(|i| {
					self.new_invariant_instr.insert(i.get_write().unwrap().clone(), i)
				});
				let (tmp2, instr) = compute_two_value(
					tmp1,
					step.clone(),
					llvm::ArithOp::Add,
					self.temp_mgr,
				);
				instr.map(|i| {
					self.new_invariant_instr.insert(i.get_write().unwrap().clone(), i)
				});
				let (tmp3, instr) = compute_two_value(
					tmp2,
					llvm::Value::Int(1),
					llvm::ArithOp::Sub,
					self.temp_mgr,
				);
				instr.map(|i| {
					self.new_invariant_instr.insert(i.get_write().unwrap().clone(), i)
				});
				let (tmp4, instr) = compute_two_value(
					tmp3,
					step.clone(),
					llvm::ArithOp::Div,
					self.temp_mgr,
				);
				instr.map(|i| {
					self.new_invariant_instr.insert(i.get_write().unwrap().clone(), i)
				});
				tmp4
			}
			CompOp::SLE | CompOp::SGE => {
				// (end - start + step) / step
				let (tmp1, instr) = compute_two_value(
					end.clone(),
					start.clone(),
					llvm::ArithOp::Sub,
					self.temp_mgr,
				);
				instr.map(|i| {
					self.new_invariant_instr.insert(i.get_write().unwrap().clone(), i)
				});
				let (tmp2, instr) = compute_two_value(
					tmp1,
					step.clone(),
					llvm::ArithOp::Add,
					self.temp_mgr,
				);
				instr.map(|i| {
					self.new_invariant_instr.insert(i.get_write().unwrap().clone(), i)
				});
				let (tmp3, instr) = compute_two_value(
					tmp2,
					step.clone(),
					llvm::ArithOp::Div,
					self.temp_mgr,
				);
				instr.map(|i| {
					self.new_invariant_instr.insert(i.get_write().unwrap().clone(), i)
				});
				tmp3
			}
			_ => unreachable!(),
		}
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
	// 把计算某 temp 的语句放入 preheader,并循 use-def 链把所有还没有放入 preheader 中的语句全都放进去
	pub fn place_temp_into_cfg(&mut self, temp: &LlvmTemp) {
		let mut work = VecDeque::new();
		work.push_back(temp.clone());
		let mut instrs = Vec::new();
		while let Some(t) = work.pop_front() {
			if self.new_invariant_instr.contains_key(&t) {
				let instr = self.new_invariant_instr.remove(&t).unwrap();
				for use_ in instr.get_read() {
					if self.new_invariant_instr.contains_key(&use_) {
						work.push_back(use_);
					}
				}
				instrs.push(instr);
			}
		}
		instrs.reverse();
		for instr in instrs.into_iter() {
			self.place_one_instr(instr);
		}
	}
	// TODO: 还没有一步到位，把生成的指令放在尽可能高的地方
	pub fn place_one_instr(&mut self, instr: LlvmInstr) {
		let mut loop_to_insert = self.cur_loop.clone();
		while loop_to_insert.borrow().outer.is_some() {
			let outer = loop_to_insert.borrow().outer.clone().unwrap();
			let outer = outer.upgrade().unwrap();
			if instr
				.get_read()
				.iter()
				.any(|t| outer.borrow().is_super_loop_of(&self.def_loop(t)))
			{
				break;
			}
			loop_to_insert = outer;
		}
		let preheader = loop_to_insert
			.borrow()
			.get_loop_preheader(&self.loopdata.loop_map)
			.unwrap_or(self.func.cfg.get_entry());
		self
			.loopdata
			.def_map
			.insert(instr.get_write().unwrap().clone(), preheader.clone());
		self
			.loopdata
			.temp_graph
			.add_temp(instr.get_write().unwrap().clone(), instr.clone());
		preheader.borrow_mut().instrs.push(instr);
		self.flag = true;
	}
	pub fn get_cur_loop_preheader(&self) -> LlvmNode {
		self.cur_loop.borrow().get_loop_preheader(&self.loopdata.loop_map).unwrap()
	}
	pub fn header_of_temp(&self, temp: &LlvmTemp) -> LlvmTemp {
		self.header_map.get(temp).cloned().unwrap_or(temp.clone())
	}
	pub fn scc_of_temp(&self, temp: &LlvmTemp) -> Vec<LlvmTemp> {
		if let Some(header) = self.header_map.get(temp) {
			self.header_map_rev[header].clone()
		} else {
			vec![temp.clone()]
		}
	}
	pub fn reads_of_temp_in_scc(&self, temp: &LlvmTemp) -> Vec<LlvmTemp> {
		let mut reads = Vec::new();
		for t in self.scc_of_temp(temp) {
			reads.extend(self.loopdata.temp_graph.get_use_temps(&t));
		}
		reads
	}
}
