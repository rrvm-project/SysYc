use crate::loops::indvar::IndVar;

use super::OneLoopSolver;

use llvm::{ArithOp, LlvmInstrVariant, LlvmTemp, Value};
impl<'a: 'b, 'b> OneLoopSolver<'a, 'b> {
	// 判断单成员的 scc 是否是循环变量
	pub fn classify_single_member_scc(&mut self, temp: &LlvmTemp) {
		// 看看它是不是归纳变量的计算结果
		let instr = self.opter.temp_graph.temp_to_instr[temp].instr.get_variant();
		match instr {
			LlvmInstrVariant::ArithInstr(inst) => {
				if let Some(iv1) = self.is_indvar(&inst.lhs) {
					if let Some(iv2) = self.is_indvar(&inst.rhs) {
						if let Some(output_iv) = self.compute_two_indvar(iv1, iv2, inst.op)
						{
							eprintln!("OneLoopSolver: computed indvar: {} base: {}, scale: {}, step: {}", temp, output_iv.base, output_iv.scale, output_iv.step);
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
							eprintln!("OneLoopSolver: computed indvar: {} base: {}, scale: {}, step: {}", temp, output_iv.base, output_iv.scale, output_iv.step);
							self.indvars.insert(temp.clone(), output_iv);
						}
					}
				}
			}
			// TODO: CompInstr
			_ => {}
		}
	}
	// 找到 scc 中在 header 中的 phi 语句
	pub fn find_header_for_scc(&mut self, scc: &Vec<LlvmTemp>) -> LlvmTemp {
		for temp in scc {
			if self.opter.temp_graph.is_phi(temp)
				&& self.opter.def_map[temp].borrow().id
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
		self.opter.def_map.get(temp).map_or(false, |bb| {
			self
				.opter
				.loop_map
				.get(&bb.borrow().id)
				.map_or(false, |l| l.borrow().id == self.cur_loop.borrow().id)
		})
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
			Value::Temp(temp) => self.opter.def_map.get(temp).map_or(true, |bb| {
				!self
					.cur_loop
					.borrow()
					.is_super_loop_of(&self.opter.loop_map[&bb.borrow().id])
			}),
			Value::Int(_) => true,
			Value::Float(_) => true,
		}
	}
}
