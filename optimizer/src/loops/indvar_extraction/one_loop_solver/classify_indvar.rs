use llvm::{compute_two_value, ArithOp, LlvmInstrVariant, LlvmTemp, Value};

use crate::loops::{chain_node::ChainNode, indvar::IndVar};

use super::OneLoopSolver;

impl<'a> OneLoopSolver<'a> {
	pub fn run(&mut self, start: LlvmTemp) {
		self.tarjan(start);
	}
	pub fn tarjan(&mut self, temp: LlvmTemp) {
		self.tarjan_var.visited.insert(temp.clone());
		self.tarjan_var.dfsnum.insert(temp.clone(), self.tarjan_var.next_dfsnum);
		self.tarjan_var.low.insert(temp.clone(), self.tarjan_var.next_dfsnum);
		self.tarjan_var.next_dfsnum += 1;
		self.stack_push(temp.clone());
		let mut reads: Vec<LlvmTemp> =
			self.loopdata.temp_graph.get_use_temps(&temp);
		// 只保留在当前循环中的变量
		reads.retain(|t| self.is_temp_in_current_loop(t));

		reads.iter().for_each(|operand| {
			if !self.tarjan_var.visited.contains(operand) {
				self.tarjan(operand.clone());
				self.tarjan_var.low.insert(
					temp.clone(),
					self.tarjan_var.low[&temp].min(self.tarjan_var.low[operand]),
				);
			} else if self.tarjan_var.dfsnum[operand] < self.tarjan_var.dfsnum[&temp] // TODO: Tarjan 算法中 这里需不需要判断 dfsnum 的大小，我感觉不用
                && self.stack_contains(operand)
			{
				self.tarjan_var.low.insert(
					temp.clone(),
					self.tarjan_var.low[&temp].min(self.tarjan_var.dfsnum[operand]),
				);
			}
		});
		if self.tarjan_var.dfsnum[&temp] == self.tarjan_var.low[&temp] {
			let mut scc = Vec::new();
			while let Some(top) = self.stack_pop() {
				scc.push(top.clone());
				if top == temp {
					break;
				}
			}
			// 检查是否是归纳变量
			self.process(scc);
		}
	}
	pub fn process(&mut self, scc: Vec<LlvmTemp>) {
		// 长度为 1 的 scc 不可能是一个单独的 header 中的 phi 语句
		// 因为这样的 phi 语句长成 X = phi(C, X) 的形式，会在 loop_simplify 中被简化掉
		// 但它有可能是一个只依赖于循环不变量的 phi 语句，而这是循环变量
		// 所以需要检查它是不是 phi 语句，以及检查它的 use 有没有循环变量。如果没有，则它一定是循环不变量，而且我们可以直接把它 append 到 preheader 中
		// Tarjan 算法发现 scc 的顺序保证了 append 的时候循环不变量不会先 use 再 def
		// TODO: 还要检查它是不是归纳变量
		if scc.len() == 1 {
			self.classify_single_member_scc(&scc[0]);
		} else {
			// 获得 scc 在 header 中的 phi 语句的 target
			let header = self.find_header_for_scc(&scc);
			scc.iter().for_each(|t| {
				self.header_map.insert(t.clone(), header.clone());
			});
			self.classify_many_members_scc(header);
		}
	}
	pub fn classify_many_members_scc(&mut self, header: LlvmTemp) {
		let mut is_zfp = None;
		let reads = self.loopdata.temp_graph.get_use_values(&header);
		assert!(reads.len() == 2);
		let (variant, phi_base) = self.get_variant_and_step(&reads[0], &reads[1], &header).expect("header of a scc must have a operand of indvar and a operand of a invariant");
		assert!(phi_base.len() == 1);
		let phi_base = phi_base[0].clone();
		let mut indvar_chain: Vec<ChainNode> = vec![];

		let mut chain_runner = variant;
		// 允许最后一个操作是 mod. 由于我这里是逆向访问 chain, 所以需要检查第一个被访问的操作是不是 mod
		if self.loopdata.temp_graph.is_mod(&chain_runner) {
			let reads = self.loopdata.temp_graph.get_use_values(&chain_runner);
			if let Some((variant, step)) =
				self.get_variant_and_step(&reads[0], &reads[1], &header)
			{
				if step.len() != 1 {
					return;
				}
				assert!(self.header_map[&variant] == header);
				indvar_chain.push(ChainNode::new(
					chain_runner,
					ArithOp::Rem,
					step.clone(),
				));
				let step = step[0].clone();
				chain_runner = variant.clone();
				is_zfp = Some(step);
			} else {
				return;
			}
		}
		while chain_runner != header {
			let reads = self.loopdata.temp_graph.get_use_values(&chain_runner);
			// For it to be on a chain, it must have at least one read
			assert!(!reads.is_empty());
			// TODO: 目前只允许 chain 中的操作是整数加法，减法，乘法
			if let Some(chain_op) =
				self.loopdata.temp_graph.is_candidate_operator(&chain_runner)
			{
				if let Some((variant, step)) =
					self.get_variant_and_step(&reads[0], &reads[1], &header)
				{
					assert!(self.header_map[&variant] == header);
					indvar_chain.push(ChainNode::new(chain_runner, chain_op, step));
					chain_runner = variant.clone();
				} else {
					return;
				}
			} else {
				return;
			}
		}
		self.compute_indvar(indvar_chain, header, phi_base, is_zfp);
	}

	// 判断单成员的 scc 是否是循环变量
	pub fn classify_single_member_scc(&mut self, temp: &LlvmTemp) {
		// 看看它是不是归纳变量的计算结果
		let instr =
			self.loopdata.temp_graph.temp_to_instr[temp].instr.get_variant();
		match instr {
			LlvmInstrVariant::ArithInstr(inst) => {
				if let Some(iv1) = self.is_indvar(&inst.lhs) {
					if let Some(iv2) = self.is_indvar(&inst.rhs) {
						if let Some(output_iv) = self.compute_two_indvar(iv1, iv2, inst.op)
						{
							// #[cfg(feature = "debug")]
							eprintln!(
								"OneLoopSolver: computed indvar: {} {} \n which is defined as {}",
								temp, output_iv, self.loopdata.temp_graph.temp_to_instr[temp].instr
							);
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
							// #[cfg(feature = "debug")]
							eprintln!(
								"OneLoopSolver: computed indvar: {} {} \n which is defined as {}",
								temp, output_iv, self.loopdata.temp_graph.temp_to_instr[temp].instr
							);
							self.indvars.insert(temp.clone(), output_iv);
						}
					}
				}
			}
			// TODO: CompInstr
			_ => {}
		}
	}
	// 根据 chain 上的内容，为一阶归纳变量计算 base, scale, step，记录在 self.indvars 中
	pub fn compute_indvar(
		&mut self,
		mut indvar_chain: Vec<ChainNode>,
		header: LlvmTemp,
		phi_base: Value,
		is_zfp: Option<Value>,
	) {
		// 从 use-def 链的父子顺序变成基本块中的语句顺序
		assert!(!indvar_chain.is_empty());
		indvar_chain.reverse();
		let mut end = indvar_chain.len();
		if is_zfp.is_some() {
			end -= 1;
		}
		let mut step = vec![Value::Int(0)];
		let mut scale = Value::Int(1);
		let mut base = phi_base.clone();
		let mut instr;
		let mut indvar_bases = vec![];
		for chain_node in indvar_chain[..end].iter() {
			match chain_node.op {
				ArithOp::Add | ArithOp::Sub => {
					// 有 + - 时保证 scale 是 1
					if scale == Value::Int(1) {
						step = self.compute_two_vec_values(
							&step,
							&chain_node.operand,
							chain_node.op,
						);
						for o in chain_node.operand.iter() {
							(base, instr) =
								compute_two_value(base, o.clone(), ArithOp::Add, self.temp_mgr);
							instr.map(|i| {
								self
									.new_invariant_instr
									.insert(i.get_write().unwrap().clone(), i)
							});
						}
						indvar_bases.push(base.clone());
					} else {
						return;
					}
				}
				ArithOp::Mul => {
					// 有 * 时保证 step 是 1 阶归纳变量
					if step.len() == 1 && chain_node.operand.len() == 1 {
						(scale, instr) = compute_two_value(
							scale,
							chain_node.operand[0].clone(),
							ArithOp::Mul,
							self.temp_mgr,
						);
						instr.map(|i| {
							self.new_invariant_instr.insert(i.get_write().unwrap().clone(), i)
						});
						(base, instr) = compute_two_value(
							base,
							chain_node.operand[0].clone(),
							ArithOp::Mul,
							self.temp_mgr,
						);
						instr.map(|i| {
							self.new_invariant_instr.insert(i.get_write().unwrap().clone(), i)
						});
						(step[0], instr) = compute_two_value(
							step[0].clone(),
							chain_node.operand[0].clone(),
							ArithOp::Mul,
							self.temp_mgr,
						);
						instr.map(|i| {
							self.new_invariant_instr.insert(i.get_write().unwrap().clone(), i)
						});
						indvar_bases.push(base.clone());
					}
				}
				_ => unreachable!(),
			}
		}
		let iv = IndVar::new(phi_base, scale.clone(), step.clone(), is_zfp.clone());
		// #[cfg(feature = "debug")]
		eprintln!("OneLoopSolver: found a indvar {} {}", header, iv);
		self.indvars.insert(header.clone(), iv);
		for (indvar, indvar_base) in indvar_chain.iter().zip(indvar_bases) {
			let iv =
				IndVar::new(indvar_base, scale.clone(), step.clone(), is_zfp.clone());
			// #[cfg(feature = "debug")]
			eprintln!(
				"OneLoopSolver: a indvar in chain {} {}",
				indvar.temp.clone(),
				iv
			);
			self.indvars.insert(indvar.temp.clone(), iv);
		}
	}
}
