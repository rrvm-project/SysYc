use std::collections::{HashSet, VecDeque};

use llvm::{
	compute_two_value, ArithInstr, ArithOp, LlvmInstrVariant, LlvmTemp, Value,
};

use super::OneLoopSolver;

use utils::UseTemp;

impl<'a> OneLoopSolver<'a> {
	pub fn indvar_extraction(&mut self) {
		if let Some(info) = self.get_loop_info() {
			self.classify_usefulness();
			let loop_cnt = self.compute_loop_cnt(&info);
			let mut headers_to_remove = HashSet::new();
			for header in info.header.borrow().phi_instrs.iter() {
				if !self.useful_variants.contains(&header.target) {
					if let Some(v) =
						self.extract_one_indvar(header.target.clone(), loop_cnt.clone())
					{
						let new_header = ArithInstr {
							target: header.target.clone(),
							op: ArithOp::Add,
							var_type: header.var_type,
							lhs: v.clone(),
							rhs: Value::Int(0),
						};
						self
							.new_invariant_instr
							.insert(header.target.clone(), Box::new(new_header));
						self.place_temp_into_cfg(&header.target);
						headers_to_remove.insert(header.target.clone());
					}
				} else {
					// #[cfg(feature = "debug")]
					eprintln!("not extract indvar: {}", header.target);
				}
			}
			info
				.header
				.borrow_mut()
				.phi_instrs
				.retain(|phi| !headers_to_remove.contains(&phi.target));
		}
	}
	pub fn extract_one_indvar(
		&mut self,
		header: LlvmTemp,
		loop_cnt: Value,
	) -> Option<Value> {
		let indvar = self.indvars[&header].clone();
		// TODO: 只展开 scale 为 1 的
		// TODO: 乘法改成双字乘法
		// TODO： zfp 归纳变量的初始值可能大于 p,需要展开一次循环
		if indvar.scale == Value::Int(1) {
			eprintln!(
				"extracting indvar: {} {} with loop_cnt: {}",
				header, indvar, loop_cnt
			);
			let mut compute_two_value =
				|a: &Value, b: &Value, op: ArithOp| -> Value {
					let (output, instr) =
						compute_two_value(a.clone(), b.clone(), op, self.temp_mgr);
					if let Some(instr) = instr {
						self
							.new_invariant_instr
							.insert(instr.get_write().unwrap().clone(), instr);
					}
					output
				};
			// k 的阶乘
			let mut fract = Value::Int(1);
			let mut coef = loop_cnt.clone();
			let mut sum = indvar.base.clone();
			for (index, step) in indvar.step.iter().enumerate() {
				let tmp1 = compute_two_value(&coef, step, ArithOp::Mul);
				let tmp2 = compute_two_value(&tmp1, &fract, ArithOp::Div);
				sum = compute_two_value(&sum, &tmp2, ArithOp::Add);
				fract = compute_two_value(
					&fract,
					&Value::Int(index as i32 + 2),
					ArithOp::Mul,
				);
				let cnt_minus_one = compute_two_value(
					&loop_cnt,
					&Value::Int(index as i32 + 1),
					ArithOp::Sub,
				);
				coef = compute_two_value(&coef, &cnt_minus_one, ArithOp::Mul);
			}
			if let Some(zfp) = indvar.zfp.as_ref() {
				sum = compute_two_value(&sum, zfp, ArithOp::Rem);
			}
			Some(sum)
		} else {
			#[cfg(feature = "debug")]
			eprintln!("not extract indvar: {} {}", header, indvar);
			None
		}
	}

	// 确定哪些 indvar 是有用的，不能外推的
	// 本身就有用的语句：store, call, 非 indvar header 的 phi
	pub fn classify_usefulness(&mut self) {
		let mut work = VecDeque::new();
		let blocks = self
			.cur_loop
			.borrow()
			.blocks_without_subloops(&self.func.cfg, &self.loopdata.loop_map);
		for block in blocks.iter() {
			for instr in block.borrow().phi_instrs.iter() {
				if !self.indvars.contains_key(&instr.target) {
					work.push_back(instr.target.clone());
				}
			}
			for instr in block.borrow().instrs.iter() {
				match instr.get_variant() {
					LlvmInstrVariant::StoreInstr(inst) => {
						work.extend(inst.get_read());
					}
					LlvmInstrVariant::CallInstr(inst) => {
						work.push_back(inst.target.clone());
					}
					_ => {}
				}
			}
			for instr in block.borrow().jump_instr.iter() {
				work.extend(instr.get_read());
			}
		}
		while let Some(temp) = work.pop_front() {
			self.useful_variants.insert(temp.clone());
			let instr = &self.loopdata.temp_graph.temp_to_instr[&temp].instr;
			let reads = instr
				.get_read()
				.into_iter()
				// 过滤掉不在当前循环中的变量
				.filter(|t| self.def_loop(t).borrow().id == self.cur_loop.borrow().id)
				// 按 scc 缩点
				.map(|t| self.header_map.get(&t).cloned().unwrap_or(t))
				// 过滤掉已经被标记为 useful 的变量
				.filter(|t| !self.useful_variants.contains(t))
				.collect::<Vec<LlvmTemp>>();
			work.extend(reads);
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
}
