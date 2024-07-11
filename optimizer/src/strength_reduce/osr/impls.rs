use std::collections::{HashMap, HashSet};

use llvm::{
	ArithInstr, ArithOp, ConvertInstr, HashableValue, LlvmInstrTrait, LlvmTemp,
	LlvmTempManager, Value, VarType,
};
use rrvm::{dominator::DomTree, LlvmCFG};
use utils::UseTemp;

use super::OSR;
impl OSR {
	pub fn new(cfg: &LlvmCFG, params: Vec<LlvmTemp>) -> Self {
		let dfsnum = HashMap::new();
		let mut visited = HashMap::new();
		let low = HashMap::new();
		let stack = Vec::new();
		let next_dfsnum = 0;
		let header = HashMap::new();
		let mut temp_to_instr = HashMap::new();
		for (bb_id, block) in cfg.blocks.iter().enumerate() {
			let block = block.borrow();
			for (instr_id, instr) in block.instrs.iter().enumerate() {
				instr.get_write().iter().for_each(|temp| {
					visited.insert(temp.clone(), false);
					temp_to_instr
						.insert(temp.clone(), (block.id, bb_id, instr_id, false));
				});
			}
			for (instr_id, instr) in block.phi_instrs.iter().enumerate() {
				let temp = instr.target.clone();
				visited.insert(temp.clone(), false);
				temp_to_instr.insert(temp.clone(), (block.id, bb_id, instr_id, true));
			}
		}

		let dom_tree = DomTree::new(cfg, false);

		Self {
			dfsnum,
			next_dfsnum,
			visited,
			low,
			stack,
			header,
			temp_to_instr,
			new_instr: HashMap::new(),
			flag: false,
			dominates: dom_tree.dominates,
			params,
			lstf_map: HashMap::new(),
			do_not_replace: HashSet::new(),
		}
	}
	pub fn run(&mut self, cfg: &mut LlvmCFG, mgr: &mut LlvmTempManager) {
		while self.visited.values().any(|&v| !v) {
			let temp = self.visited.iter().find(|(_, &v)| !v).unwrap().0.clone();
			self.dfs(cfg, temp, mgr);
		}
		self.lstf(cfg, mgr);
	}
	pub fn dfs(
		&mut self,
		cfg: &mut LlvmCFG,
		temp: LlvmTemp,
		mgr: &mut LlvmTempManager,
	) {
		self.visited.insert(temp.clone(), true);
		self.dfsnum.insert(temp.clone(), self.next_dfsnum);
		self.low.insert(temp.clone(), self.next_dfsnum);
		self.next_dfsnum += 1;
		self.stack.push(temp.clone());
		let mut reads = self.get_instr_reads(cfg, temp.clone());
		reads.retain(|t| !t.is_global && !self.params.contains(t));
		reads.iter().for_each(|operand| {
			if !self.visited[operand] {
				self.dfs(cfg, operand.clone(), mgr);
				self.low.insert(temp.clone(), self.low[&temp].min(self.low[operand]));
			}
			// 这里判断在不在栈上，或许可以开一个 HashMap<LlvmTemp, bool>, 实现 O(1) 的判断
			if self.dfsnum[operand] < self.dfsnum[&temp]
				&& self.stack.contains(operand)
			{
				self
					.low
					.insert(temp.clone(), self.low[&temp].min(self.dfsnum[operand]));
			}
		});
		if self.dfsnum[&temp] == self.low[&temp] {
			let mut scc = Vec::new();
			while let Some(top) = self.stack.pop() {
				scc.push(top.clone());
				if top == temp {
					break;
				}
			}
			self.process(cfg, scc, mgr);
		}
	}
	pub fn process(
		&mut self,
		cfg: &mut LlvmCFG,
		scc: Vec<LlvmTemp>,
		mgr: &mut LlvmTempManager,
	) {
		if scc.len() == 1 {
			let member = &scc[0];
			let (_, bb_id, instr_id, is_phi) =
				self.temp_to_instr.get(member).unwrap();
			if let Some((iv, rc)) =
				self.is_candidate_operation(cfg, *bb_id, *instr_id, *is_phi)
			{
				self.replace(cfg, member.clone(), iv, rc, mgr);
			} else {
				self.header.remove(member);
			}
		} else {
			self.classify_induction_variables(cfg, scc, mgr);
		}
	}
	pub fn classify_induction_variables(
		&mut self,
		cfg: &mut LlvmCFG,
		scc: Vec<LlvmTemp>,
		mgr: &mut LlvmTempManager,
	) {
		if scc.len() == 2 {
			let member1 = &scc[0];
			let member2 = &scc[1];
			if let Some((_, bb_index, instr_index, true)) =
				self.temp_to_instr.get(member2).cloned()
			{
				let src_num =
					cfg.blocks[bb_index].borrow().phi_instrs[instr_index].source.len();
				if src_num == 2
					&& self.is_valid_update_temp(cfg, member2.clone(), member1.clone())
				{
					self.header.insert(member1.clone(), member2.clone());
					self.header.insert(member2.clone(), member2.clone());

					let mut connected_temp_cnt = 0;
					let mut candidate_operators = Vec::new();
					for (instr_id, instr) in
						cfg.blocks[bb_index].borrow().instrs.iter().enumerate()
					{
						if (instr.get_read().contains(member2)
							|| instr.get_read().contains(member1))
							&& !self.is_replaceable_cmp(instr.get_lhs_and_rhs())
						{
							connected_temp_cnt += 1;
							if self
								.is_candidate_operation(cfg, bb_index, instr_id, false)
								.is_some()
							{
								candidate_operators.push(instr.get_write().unwrap());
							}
						}
					}
					if connected_temp_cnt >= 2 {
						self.do_not_replace.extend(candidate_operators);
					}
				}
			} else {
				for p in scc.iter() {
					let (_, bb_id, instr_id, is_phi) = self.temp_to_instr.get(p).unwrap();
					if let Some((iv, rc)) =
						self.is_candidate_operation(cfg, *bb_id, *instr_id, *is_phi)
					{
						self.replace(cfg, p.clone(), iv, rc, mgr);
					} else {
						self.header.remove(p);
					}
				}
			}
		} else {
			for p in scc.iter() {
				let (_, bb_id, instr_id, is_phi) = self.temp_to_instr.get(p).unwrap();
				if let Some((iv, rc)) =
					self.is_candidate_operation(cfg, *bb_id, *instr_id, *is_phi)
				{
					self.replace(cfg, p.clone(), iv, rc, mgr);
				} else {
					self.header.remove(p);
				}
			}
		}
	}
	// 对候选操作进行替换
	// phi 指令一定不是候选操作
	pub fn replace(
		&mut self,
		cfg: &mut LlvmCFG,
		scc_member: LlvmTemp,
		iv: LlvmTemp,
		rc: Value,
		mgr: &mut LlvmTempManager,
	) {
		if self.do_not_replace.contains(&scc_member) {
			return;
		}
		let (_, bb_id, instr_id, _is_phi) =
			*self.temp_to_instr.get(&scc_member).unwrap();
		let op = cfg.blocks.get(bb_id).unwrap().borrow().instrs[instr_id]
			.is_candidate_operator()
			.unwrap();
		let result = self.reduce(cfg, op, iv.clone(), rc, mgr);
		let (_, bb_id, instr_id, _is_phi) =
			*self.temp_to_instr.get(&scc_member).unwrap();
		self.replace_to_copy(cfg, bb_id, instr_id, result);
		self.flag = true;
		self.header.insert(scc_member, self.header[&iv].clone());
	}
	pub fn reduce(
		&mut self,
		cfg: &mut LlvmCFG,
		op: ArithOp,
		iv: LlvmTemp,
		rc: Value,
		mgr: &mut LlvmTempManager,
	) -> LlvmTemp {
		if let Some(t) = self.new_instr.get(&(
			op,
			HashableValue::from(Value::Temp(iv.clone())),
			HashableValue::from(rc.clone()),
		)) {
			return t.clone();
		}
		if op.is_commutative() {
			if let Some(t) = self.new_instr.get(&(
				op,
				HashableValue::from(rc.clone()),
				HashableValue::from(Value::Temp(iv.clone())),
			)) {
				return t.clone();
			}
		}
		let result = self.new_temp(iv.var_type, mgr);
		self.new_instr.insert(
			(
				op,
				HashableValue::from(Value::Temp(iv.clone())),
				HashableValue::from(rc.clone()),
			),
			result.clone(),
		);
		let (id, bb_id, instr_id, is_phi) = *self.temp_to_instr.get(&iv).unwrap();
		if is_phi {
			let mut new_def = cfg.blocks[bb_id].borrow().phi_instrs[instr_id].clone();
			self.add_lstf_edge(
				new_def.target.clone(),
				result.clone(),
				op,
				rc.clone(),
			);
			new_def.swap_target(result.clone());

			cfg.blocks[bb_id].borrow_mut().phi_instrs[((instr_id) + 1)..]
				.iter()
				.for_each(|i| {
					i.get_write().iter().for_each(|t| {
						self.temp_to_instr.entry(t.clone()).and_modify(
							|(_, _, instr_id, _)| {
								*instr_id += 1;
							},
						);
					})
				});
			cfg.blocks[bb_id].borrow_mut().phi_instrs.insert(instr_id + 1, new_def);
			self.flag = true;

			self
				.temp_to_instr
				.insert(result.clone(), (id, bb_id, instr_id + 1, true));
			// self.dfsnum.insert(result.clone(), self.dfsnum[&iv]);
			// self.visited.insert(result.clone(), self.visited[&iv]);
			// self.low.insert(result.clone(), self.low[&iv]);
			self.header.insert(result.clone(), self.header[&iv].clone());

			let new_def_read_values = cfg.blocks[bb_id].borrow_mut().phi_instrs
				[instr_id + 1]
				.get_read_values();
			for (id, operand) in new_def_read_values.iter().enumerate() {
				if operand.unwrap_temp().is_some_and(|t| {
					self.header.get(&t).is_some_and(|h| *h == self.header[&iv])
				}) {
					let t = operand.unwrap_temp().unwrap();
					let new_value =
						Value::Temp(self.reduce(cfg, op, t.clone(), rc.clone(), mgr));
					// 重新获得一次语句的位置
					let (_, bb_id, instr_id, _) =
						*self.temp_to_instr.get(&result).unwrap();
					cfg.blocks[bb_id].borrow_mut().phi_instrs[instr_id]
						.set_read_values(id, new_value);
				} else {
					let new_value = self.apply(cfg, op, operand.clone(), rc.clone(), mgr);
					let (_, bb_id, instr_id, _) =
						*self.temp_to_instr.get(&result).unwrap();
					cfg.blocks[bb_id].borrow_mut().phi_instrs[instr_id]
						.set_read_values(id, new_value);
				}
			}
		} else {
			let mut new_def = cfg.blocks[bb_id].borrow().instrs[instr_id].clone_box();
			self.add_lstf_edge(
				new_def.get_write().unwrap(),
				result.clone(),
				op,
				rc.clone(),
			);
			new_def.swap_target(result.clone());

			cfg.blocks[bb_id].borrow_mut().instrs[((instr_id) + 1)..]
				.iter()
				.for_each(|i| {
					i.get_write().iter().for_each(|t| {
						self.temp_to_instr.entry(t.clone()).and_modify(
							|(_, _, instr_id, _)| {
								*instr_id += 1;
							},
						);
					})
				});
			cfg.blocks[bb_id].borrow_mut().instrs.insert(instr_id + 1, new_def);
			self.flag = true;

			self
				.temp_to_instr
				.insert(result.clone(), (id, bb_id, instr_id + 1, false));
			// self.dfsnum.insert(result.clone(), self.dfsnum[&iv]);
			// self.visited.insert(result.clone(), self.visited[&iv]);
			// self.low.insert(result.clone(), self.low[&iv]);
			self.header.insert(result.clone(), self.header[&iv].clone());

			let new_def_read_values =
				cfg.blocks[bb_id].borrow_mut().instrs[instr_id + 1].get_read_values();
			for (id, operand) in new_def_read_values.iter().enumerate() {
				if let Some(t) = operand.unwrap_temp() {
					if self.header.get(&t).is_some_and(|h| *h == self.header[&iv]) {
						let new_value =
							Value::Temp(self.reduce(cfg, op, t.clone(), rc.clone(), mgr));
						// 重新获得一次语句的位置
						let (_, bb_id, instr_id, _) =
							*self.temp_to_instr.get(&result).unwrap();
						cfg.blocks[bb_id].borrow_mut().instrs[instr_id]
							.set_read_values(id, new_value);
					}
				} else if op == ArithOp::Mul || op == ArithOp::Fmul {
					let new_value = self.apply(cfg, op, operand.clone(), rc.clone(), mgr);
					let (_, bb_id, instr_id, _) =
						*self.temp_to_instr.get(&result).unwrap();
					cfg.blocks[bb_id].borrow_mut().instrs[instr_id]
						.set_read_values(id, new_value);
				}
			}
		}
		result
	}
	pub fn apply(
		&mut self,
		cfg: &mut LlvmCFG,
		op: ArithOp,
		operand1: Value,
		operand2: Value,
		mgr: &mut LlvmTempManager,
	) -> Value {
		if let Some(t) = self.new_instr.get(&(
			op,
			HashableValue::from(operand1.clone()),
			HashableValue::from(operand2.clone()),
		)) {
			return Value::Temp(t.clone());
		}
		if op.is_commutative() {
			if let Some(t) = self.new_instr.get(&(
				op,
				HashableValue::from(operand2.clone()),
				HashableValue::from(operand1.clone()),
			)) {
				return Value::Temp(t.clone());
			}
		}
		if let Some((iv, header)) = self.is_induction_value(operand1.clone()) {
			if let Some(rc) =
				self.is_regional_constant(header.clone(), operand2.clone())
			{
				let result = self.reduce(cfg, op, iv.clone(), rc, mgr);
				return Value::Temp(result);
			}
		}
		if let Some((iv, header)) = self.is_induction_value(operand2.clone()) {
			if let Some(rc) =
				self.is_regional_constant(header.clone(), operand1.clone())
			{
				let result = self.reduce(cfg, op, iv.clone(), rc, mgr);
				return Value::Temp(result);
			}
		}
		let result;
		let bb_id_to_insert;
		let bb_index_to_insert;

		match (&operand1, &operand2) {
			(Value::Temp(t1), Value::Temp(t2)) => {
				if self.params.contains(t1) && self.params.contains(t2) {
					bb_id_to_insert = cfg.get_entry().borrow().id;
					bb_index_to_insert = 0;
				} else if self.params.contains(t1) {
					let (t2_id, t2_bb_index, _, _) = *self.temp_to_instr.get(t2).unwrap();
					bb_id_to_insert = t2_id;
					bb_index_to_insert = t2_bb_index;
				} else if self.params.contains(t2) {
					let (t1_id, t1_bb_index, _, _) = *self.temp_to_instr.get(t1).unwrap();
					bb_id_to_insert = t1_id;
					bb_index_to_insert = t1_bb_index;
				} else {
					let (t1_id, t1_bb_index, _, _) = *self.temp_to_instr.get(t1).unwrap();
					let (t2_id, t2_bb_index, _, _) = *self.temp_to_instr.get(t2).unwrap();
					if self.dominates[&t1_id].iter().any(|bb| bb.borrow().id == t2_id) {
						bb_id_to_insert = t2_id;
						bb_index_to_insert = t2_bb_index;
					} else {
						bb_id_to_insert = t1_id;
						bb_index_to_insert = t1_bb_index;
					}
				}
			}
			(Value::Temp(t), _) | (_, Value::Temp(t)) => {
				if self.params.contains(t) {
					bb_id_to_insert = cfg.get_entry().borrow().id;
					bb_index_to_insert = 0;
				} else {
					let (t_id, t_bb_index, _, _) = *self.temp_to_instr.get(t).unwrap();
					bb_id_to_insert = t_id;
					bb_index_to_insert = t_bb_index;
				}
			}
			(Value::Int(i1), Value::Int(i2)) => match op {
				ArithOp::Add | ArithOp::Fadd => {
					return Value::Int(i1 + i2);
				}
				ArithOp::Sub | ArithOp::Fsub => {
					return Value::Int(i1 - i2);
				}
				ArithOp::Mul | ArithOp::Fmul => {
					return Value::Int(i1 * i2);
				}
				ArithOp::Div | ArithOp::Fdiv => {
					return Value::Int(i1 / i2);
				}
				_ => unreachable!(),
			},
			(Value::Float(f1), Value::Float(f2)) => match op {
				ArithOp::Add | ArithOp::Fadd => {
					return Value::Float(f1 + f2);
				}
				ArithOp::Sub | ArithOp::Fsub => {
					return Value::Float(f1 - f2);
				}
				ArithOp::Mul | ArithOp::Fmul => {
					return Value::Float(f1 * f2);
				}
				ArithOp::Div | ArithOp::Fdiv => {
					return Value::Float(f1 / f2);
				}
				_ => unreachable!(),
			},
			(Value::Int(i1), Value::Float(f1)) => match op {
				ArithOp::Add | ArithOp::Fadd => {
					return Value::Float(*i1 as f32 + f1);
				}
				ArithOp::Sub | ArithOp::Fsub => {
					return Value::Float(*i1 as f32 - f1);
				}
				ArithOp::Mul | ArithOp::Fmul => {
					return Value::Float(*i1 as f32 * f1);
				}
				ArithOp::Div | ArithOp::Fdiv => {
					return Value::Float(*i1 as f32 / f1);
				}
				_ => unreachable!(),
			},
			(Value::Float(f1), Value::Int(i1)) => match op {
				ArithOp::Add | ArithOp::Fadd => {
					return Value::Float(f1 + *i1 as f32);
				}
				ArithOp::Sub | ArithOp::Fsub => {
					return Value::Float(f1 - *i1 as f32);
				}
				ArithOp::Mul | ArithOp::Fmul => {
					return Value::Float(f1 * *i1 as f32);
				}
				ArithOp::Div | ArithOp::Fdiv => {
					return Value::Float(f1 / *i1 as f32);
				}
				_ => unreachable!(),
			},
		};
		self.flag = true;
		match (operand1.get_type(), operand2.get_type()) {
			(VarType::I32, VarType::I32) => {
				result = self.new_temp(VarType::I32, mgr);
				self.new_instr.insert(
					(
						op,
						HashableValue::from(operand1.clone()),
						HashableValue::from(operand2.clone()),
					),
					result.clone(),
				);
				let new_instr = ArithInstr {
					target: result.clone(),
					op: op.to_int_op(),
					var_type: VarType::I32,
					lhs: operand1,
					rhs: operand2,
				};
				cfg.blocks[bb_index_to_insert]
					.borrow_mut()
					.instrs
					.push(Box::new(new_instr));

				let instr_len = cfg.blocks[bb_index_to_insert].borrow().instrs.len();
				self.temp_to_instr.insert(
					result.clone(),
					(bb_id_to_insert, bb_index_to_insert, instr_len, false),
				);
			}
			(VarType::I32, VarType::F32) => {
				result = self.new_temp(VarType::F32, mgr);

				let convert_result = self.new_temp(VarType::F32, mgr);
				let new_convert_instr = ConvertInstr {
					target: convert_result.clone(),
					op: llvm::ConvertOp::Int2Float,
					lhs: operand1,
					var_type: VarType::F32,
				};

				let new_instr = ArithInstr {
					target: result.clone(),
					op: op.to_float_op(),
					var_type: VarType::F32,
					lhs: Value::Temp(convert_result.clone()),
					rhs: operand2,
				};

				let instr_len = cfg.blocks[bb_index_to_insert].borrow().instrs.len();
				cfg.blocks[bb_index_to_insert]
					.borrow_mut()
					.instrs
					.push(Box::new(new_convert_instr));
				cfg.blocks[bb_index_to_insert]
					.borrow_mut()
					.instrs
					.push(Box::new(new_instr));

				self.temp_to_instr.insert(
					convert_result.clone(),
					(bb_id_to_insert, bb_index_to_insert, instr_len, false),
				);
				self.temp_to_instr.insert(
					result.clone(),
					(bb_id_to_insert, bb_index_to_insert, instr_len + 1, false),
				);
			}
			(VarType::F32, VarType::I32) => {
				result = self.new_temp(VarType::F32, mgr);

				let convert_result = self.new_temp(VarType::F32, mgr);
				let new_convert_instr = ConvertInstr {
					target: convert_result.clone(),
					op: llvm::ConvertOp::Int2Float,
					var_type: VarType::F32,
					lhs: operand2,
				};

				let new_instr = ArithInstr {
					target: result.clone(),
					op: op.to_float_op(),
					var_type: VarType::F32,
					lhs: operand1,
					rhs: Value::Temp(convert_result.clone()),
				};

				let instr_len = cfg.blocks[bb_index_to_insert].borrow().instrs.len();
				cfg.blocks[bb_index_to_insert]
					.borrow_mut()
					.instrs
					.push(Box::new(new_convert_instr));
				cfg.blocks[bb_index_to_insert]
					.borrow_mut()
					.instrs
					.push(Box::new(new_instr));

				self.temp_to_instr.insert(
					convert_result.clone(),
					(bb_id_to_insert, bb_index_to_insert, instr_len, false),
				);
				self.temp_to_instr.insert(
					result.clone(),
					(bb_id_to_insert, bb_index_to_insert, instr_len + 1, false),
				);
			}
			(VarType::F32, VarType::F32) => {
				result = self.new_temp(VarType::F32, mgr);
				let new_instr = ArithInstr {
					target: result.clone(),
					op: op.to_float_op(),
					var_type: VarType::F32,
					lhs: operand1,
					rhs: operand2,
				};
				let instr_len = cfg.blocks[bb_index_to_insert].borrow().instrs.len();
				cfg.blocks[bb_index_to_insert]
					.borrow_mut()
					.instrs
					.push(Box::new(new_instr));
				self.temp_to_instr.insert(
					result.clone(),
					(bb_id_to_insert, bb_index_to_insert, instr_len, false),
				);
			}
			_ => unreachable!(),
		}
		Value::Temp(result)
	}
}
