use crate::indvar::IndVar;

use super::IndVarSolver;

use llvm::{ArithOp, LlvmInstrVariant, LlvmTemp, Value};
impl<'a> IndVarSolver<'a> {
	pub fn stack_push(&mut self, temp: LlvmTemp) {
		self.stack.push(temp.clone());
		self.in_stack.insert(temp);
	}
	pub fn stack_pop(&mut self) -> Option<LlvmTemp> {
		let temp = self.stack.pop();
		if let Some(t) = &temp {
			self.in_stack.remove(t);
		}
		temp
	}
	pub fn stack_contains(&self, temp: &LlvmTemp) -> bool {
		self.in_stack.contains(temp)
	}
	// 判断单成员的 scc 是否是循环变量
	pub fn classify_single_member_scc(&mut self, temp: &LlvmTemp) -> bool {
		// 函数参数一定是循环不变量
		if self.params.contains(temp) {
			return false;
		}
		// 定义在循环外的变量一定是循环不变量
		if !self.cur_loop.borrow().is_super_loop_of(&self.loop_map[&self.def_map[temp].borrow().id]) {
			return false;
		}
		if self.temp_graph.is_phi(temp) {
			return true;
		}
		if self.temp_graph.is_call(temp) {
			return true;
		}
		if self.temp_graph.is_load(temp) {
			return true;
		}
		// 使用的变量都是循环不变量
		if self
			.temp_graph
			.get_use_temps(temp)
			.iter()
			.all(|t| self.is_loop_invariant(&Value::Temp(t.clone())))
		{
			return false;
		}
		// 使用的变量有在内层循环的
		if self.temp_graph.get_use_temps(temp).iter().any(|t| {
			self.def_map.get(t).is_some_and(|bb| {
				self
					.cur_loop
					.borrow()
					.is_strict_super_loop_of(&self.loop_map[&bb.borrow().id])
			})
		}) {
			return true;
		}
		// 此后，它就一定是循环变量了，但需要看看它是不是归纳变量的计算结果
		match self.temp_graph.temp_to_instr[temp].instr.get_variant() {
			LlvmInstrVariant::ArithInstr(inst) => {
				if let Some(iv1) = self.is_indvar(&inst.lhs) {
					if let Some(iv2) = self.is_indvar(&inst.rhs) {
						if let Some(output_iv) = self.compute_two_indvar(iv1, iv2, inst.op)
						{
							eprintln!("IndVarSolver: computed indvar: {} base: {}, scale: {}, step: {}", temp, output_iv.base, output_iv.scale, output_iv.step);
							self.indvars.insert(temp.clone(), output_iv);
						}
					}
				}
			}
			LlvmInstrVariant::GEPInstr(inst) => {
				if let Some(iv1) = self.is_indvar(&inst.addr) {
					if let Some(iv2) = self.is_indvar(&inst.offset) {
						if let Some(output_iv) =
							self.compute_two_indvar(iv1, iv2, ArithOp::Add)
						{
							eprintln!("IndVarSolver: computed indvar: {} base: {}, scale: {}, step: {}", temp, output_iv.base, output_iv.scale, output_iv.step);
							self.indvars.insert(temp.clone(), output_iv);
						}
					}
				}
			}
			// TODO: CompInstr
			_ => {}
		}
		true
	}
	// 找到 scc 中在 header 中的 phi 语句
	pub fn find_header_for_scc(&mut self, scc: &Vec<LlvmTemp>) -> LlvmTemp {
		for temp in scc {
			if self.temp_graph.is_phi(temp)
				&& self.def_map[temp].borrow().id
					== self.cur_loop.borrow().header.borrow().id
			{
				return temp.clone();
			}
		}
		unreachable!()
	}
	pub fn get_variant_and_step(
		&self,
		member1: &Value,
		member2: &Value,
		header: &LlvmTemp,
	) -> Option<(LlvmTemp, Value)> {
		let get_variant_and_step_inner =
			|m1: &Value, m2: &Value| -> Option<(LlvmTemp, Value)> {
				if let Some(t) = m1.unwrap_temp() {
					if self.header_map.get(&t).is_some_and(|t| t == header)
						&& self.is_loop_invariant(m2)
					{
						// TODO: 暂时不允许 step 是归纳变量，也就是暂时先不考虑高阶归纳变量
						return Some((t, m2.clone()));
					}
				}
				None
			};

		get_variant_and_step_inner(member1, member2)
			.or_else(|| get_variant_and_step_inner(member2, member1))
	}
	pub fn is_temp_in_current_loop(&self, temp: &LlvmTemp) -> bool {
		!self.params.contains(temp)
			&& !temp.is_global
			&& self
				.loop_map
				.get(&self.def_map[temp].borrow().id)
				.is_some_and(|l| l.borrow().id == self.cur_loop.borrow().id)
	}
	pub fn is_indvar(&self, value: &Value) -> Option<IndVar> {
		self
			.is_loop_invariant(value)
			.then(|| IndVar::from_loop_invariant(value.clone()))
			.or(value.unwrap_temp().and_then(|temp| self.indvars.get(&temp).cloned()))
	}
	pub fn is_loop_invariant(&self, value: &Value) -> bool {
		match value {
			Value::Temp(temp) => {
				self.loop_invariant.contains(temp)
					|| self.params.contains(temp)
					|| temp.is_global
					|| !self.cur_loop.borrow().is_super_loop_of(&self.loop_map[&self.def_map[temp].borrow().id])
			}
			Value::Int(_) => true,
			Value::Float(_) => true,
		}
	}
}
