// Ref：Engineering a Compiler 2nd Edition Page 433
mod helper_functions;
use std::collections::{HashMap, VecDeque};

use llvm::{
	ArithInstr, ArithOp, ConvertInstr, HashableValue, LlvmInstrTrait, Temp,
	Value, VarType,
};
use rrvm::{dominator::naive::compute_dominator, LlvmCFG, LlvmNode};
use utils::UseTemp;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InductionVariableState {
	Valid,
	Unknown,
}

#[allow(clippy::upper_case_acronyms)]
pub struct OSR {
	// dfs 过程中，访问到的次序
	dfsnum: HashMap<Temp, i32>,
	next_dfsnum: i32,
	visited: HashMap<Temp, bool>,
	// Tarjan 算法计算强连通分量时，需要用到的值
	low: HashMap<Temp, i32>,
	stack: Vec<Temp>,
	header: HashMap<Temp, Temp>,
	// 临时变量到（基本块id，基本块数组下标，指令数组下标，是否是 phi 指令）的映射
	temp_to_instr: HashMap<Temp, (i32, usize, usize, bool)>,

	// 记录因为候选操作而产生的指令，防止产生重复的指令
	new_instr: HashMap<(ArithOp, HashableValue, HashableValue), Temp>,
	pub total_new_temp: u32,
	// 此过程是否做出了优化
	pub flag: bool,

	dominates: HashMap<i32, Vec<LlvmNode>>,
	// dominates_directly: HashMap<i32, Vec<LlvmNode>>,
	// dominator: HashMap<i32, LlvmNode>,
}

impl OSR {
	pub fn new(cfg: &mut LlvmCFG, total_new_temp: u32) -> Self {
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

		let mut dominates = HashMap::new();
		let mut dominates_directly = HashMap::new();
		let mut dominator = HashMap::new();
		compute_dominator(
			cfg,
			false,
			&mut dominates,
			&mut dominates_directly,
			&mut dominator,
		);

		Self {
			dfsnum,
			next_dfsnum,
			visited,
			low,
			stack,
			header,
			temp_to_instr,
			// to_insert: Vec::new(),
			new_instr: HashMap::new(),
			total_new_temp,
			flag: false,
			dominates,
			// dominates_directly,
			// dominator,
		}
	}
	pub fn run(&mut self, cfg: &mut LlvmCFG) {
		while self.visited.values().any(|&v| !v) {
			let temp = self.visited.iter().find(|(_, &v)| !v).unwrap().0.clone();
			self.dfs(cfg, temp);
		}
	}
	pub fn dfs(&mut self, cfg: &mut LlvmCFG, temp: Temp) {
		self.visited.insert(temp.clone(), true);
		self.dfsnum.insert(temp.clone(), self.next_dfsnum);
		self.low.insert(temp.clone(), self.next_dfsnum);
		self.next_dfsnum += 1;
		self.stack.push(temp.clone());
		let mut reads = self.get_instr_reads(cfg, temp.clone());
		reads.retain(|t| !t.is_global);
		reads.iter().for_each(|operand| {
			if !self.visited[operand] {
				self.dfs(cfg, operand.clone());
				self.low.insert(temp.clone(), self.low[&temp].min(self.low[operand]));
			}
			// 这里判断在不在栈上，或许可以开一个 HashMap<Temp, bool>, 实现 O(1) 的判断
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
			self.process(cfg, scc);
		}
	}
	pub fn process(&mut self, cfg: &mut LlvmCFG, scc: Vec<Temp>) {
		if scc.len() == 1 {
			let member = &scc[0];
			let (_, bb_id, instr_id, is_phi) =
				self.temp_to_instr.get(member).unwrap();
			if let Some((iv, rc)) =
				self.is_candidate_operation(cfg, *bb_id, *instr_id, *is_phi)
			{
				self.replace(cfg, member.clone(), iv, rc);
			} else {
				self.header.remove(member);
			}
		} else {
			self.classify_induction_variables(cfg, scc);
		}
	}
	pub fn classify_induction_variables(
		&mut self,
		cfg: &mut LlvmCFG,
		scc: Vec<Temp>,
	) {
		let mut is_induction_variable = true;
		let mut visited = HashMap::new();
		let mut worklist = VecDeque::from(scc.clone());
		for p in scc.iter() {
			if self.header.get(p).is_some() {
				visited.insert(p.clone(), InductionVariableState::Valid);
			} else {
				visited.insert(p.clone(), InductionVariableState::Unknown);
			}
		}
		while let Some(p) = worklist.pop_front() {
			let (_, bb_id, instr_id, is_phi) = self.temp_to_instr.get(&p).unwrap();
			if *is_phi {
				visited.insert(p.clone(), InductionVariableState::Valid);
				continue;
			}
			let instr = &cfg.blocks[*bb_id].borrow().instrs[*instr_id];
			if instr.get_read().iter().any(|operand| {
				visited
					.get(operand)
					.map_or(false, |&v| v == InductionVariableState::Unknown)
			}) {
				worklist.push_back(p.clone());
			} else if let Some(op) = instr.is_candidate_operator() {
				let scc_header = scc.last().unwrap().clone();
				match op {
					ArithOp::Add | ArithOp::Fadd => {
						let (lhs, rhs) = instr.get_lhs_and_rhs().unwrap();
						if let Some((_iv, header)) = self.is_induction_value(lhs.clone()) {
							if let Some(_rc) = self.is_regional_constant(header.clone(), rhs)
							{
								visited.insert(p.clone(), InductionVariableState::Valid);
							} else {
								is_induction_variable = false;
								break;
							}
						} else if lhs.unwrap_temp().is_some_and(|t| {
							visited
								.get(&t)
								.map_or(false, |&v| v == InductionVariableState::Valid)
						}) {
							if let Some(_rc) =
								self.is_regional_constant(scc_header.clone(), rhs)
							{
								visited.insert(p.clone(), InductionVariableState::Valid);
							} else {
								is_induction_variable = false;
								break;
							}
						} else if let Some((_iv, header)) =
							self.is_induction_value(rhs.clone())
						{
							if let Some(_rc) = self.is_regional_constant(header.clone(), lhs)
							{
								visited.insert(p.clone(), InductionVariableState::Valid);
							} else {
								is_induction_variable = false;
								break;
							}
						} else if rhs.unwrap_temp().is_some_and(|t| {
							visited
								.get(&t)
								.map_or(false, |&v| v == InductionVariableState::Valid)
						}) {
							if let Some(_rc) =
								self.is_regional_constant(scc_header.clone(), lhs)
							{
								visited.insert(p.clone(), InductionVariableState::Valid);
							} else {
								is_induction_variable = false;
								break;
							}
						}
					}
					ArithOp::Sub | ArithOp::Fsub => {
						let (lhs, rhs) = instr.get_lhs_and_rhs().unwrap();
						if let Some((_iv, header)) = self.is_induction_value(lhs.clone()) {
							if let Some(_rc) = self.is_regional_constant(header.clone(), rhs)
							{
								visited.insert(p.clone(), InductionVariableState::Valid);
							} else {
								is_induction_variable = false;
								break;
							}
						} else if lhs.unwrap_temp().is_some_and(|t| {
							visited
								.get(&t)
								.map_or(false, |&v| v == InductionVariableState::Valid)
						}) {
							if let Some(_rc) =
								self.is_regional_constant(scc_header.clone(), rhs)
							{
								visited.insert(p.clone(), InductionVariableState::Valid);
							} else {
								is_induction_variable = false;
								break;
							}
						}
					}
					_ => {
						is_induction_variable = false;
						break;
					}
				}
			} else {
				is_induction_variable = false;
				break;
			}
		}
		if is_induction_variable {
			for p in scc.iter() {
				self.header.insert(p.clone(), scc.last().unwrap().clone());
			}
		} else {
			for p in scc.iter() {
				let (_, bb_id, instr_id, is_phi) = self.temp_to_instr.get(p).unwrap();
				if let Some((iv, rc)) =
					self.is_candidate_operation(cfg, *bb_id, *instr_id, *is_phi)
				{
					self.replace(cfg, p.clone(), iv, rc);
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
		scc_member: Temp,
		iv: Temp,
		rc: Value,
	) {
		let (_, bb_id, instr_id, _is_phi) =
			*self.temp_to_instr.get(&scc_member).unwrap();
		let op = cfg.blocks[bb_id].borrow().instrs[instr_id]
			.is_candidate_operator()
			.unwrap();
		let result = self.reduce(cfg, op, iv.clone(), rc);
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
		iv: Temp,
		rc: Value,
	) -> Temp {
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
		let result = self.new_temp(iv.var_type);
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
			self.dfsnum.insert(result.clone(), self.dfsnum[&iv]);
			self.visited.insert(result.clone(), self.visited[&iv]);
			self.low.insert(result.clone(), self.low[&iv]);
			self.header.insert(result.clone(), self.header[&iv].clone());

			// new_def = &mut cfg.blocks[bb_id].borrow_mut().instrs[instr_id + 1];
			let new_def_read_values = cfg.blocks[bb_id].borrow_mut().phi_instrs
				[instr_id + 1]
				.get_read_values();
			for (id, operand) in new_def_read_values.iter().enumerate() {
				if let Some(t) = operand.unwrap_temp() {
					if self.header.get(&t).is_some_and(|h| *h == self.header[&iv]) {
						let new_value =
							Value::Temp(self.reduce(cfg, op, t.clone(), rc.clone()));
						// 重新获得一次语句的位置
						let (_, bb_id, instr_id, _) =
							*self.temp_to_instr.get(&result).unwrap();
						cfg.blocks[bb_id].borrow_mut().phi_instrs[instr_id]
							.set_read_values(id, new_value);
					}
				} else {
					let new_value = self.apply(cfg, op, operand.clone(), rc.clone());
					let (_, bb_id, instr_id, _) =
						*self.temp_to_instr.get(&result).unwrap();
					cfg.blocks[bb_id].borrow_mut().phi_instrs[instr_id]
						.set_read_values(id, new_value);
				}
			}
		} else {
			let mut new_def = cfg.blocks[bb_id].borrow().instrs[instr_id].clone_box();
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
			self.dfsnum.insert(result.clone(), self.dfsnum[&iv]);
			self.visited.insert(result.clone(), self.visited[&iv]);
			self.low.insert(result.clone(), self.low[&iv]);
			self.header.insert(result.clone(), self.header[&iv].clone());

			// new_def = &mut cfg.blocks[bb_id].borrow_mut().instrs[instr_id + 1];
			let new_def_read_values =
				cfg.blocks[bb_id].borrow_mut().instrs[instr_id + 1].get_read_values();
			for (id, operand) in new_def_read_values.iter().enumerate() {
				if let Some(t) = operand.unwrap_temp() {
					if self.header.get(&t).is_some_and(|h| *h == self.header[&iv]) {
						let new_value =
							Value::Temp(self.reduce(cfg, op, t.clone(), rc.clone()));
						// 重新获得一次语句的位置
						let (_, bb_id, instr_id, _) =
							*self.temp_to_instr.get(&result).unwrap();
						cfg.blocks[bb_id].borrow_mut().instrs[instr_id]
							.set_read_values(id, new_value);
					}
				} else if op == ArithOp::Mul || op == ArithOp::Fmul {
					let new_value = self.apply(cfg, op, operand.clone(), rc.clone());
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
				let result = self.reduce(cfg, op, iv.clone(), rc);
				return Value::Temp(result);
			}
		}
		if let Some((iv, header)) = self.is_induction_value(operand2.clone()) {
			if let Some(rc) =
				self.is_regional_constant(header.clone(), operand1.clone())
			{
				let result = self.reduce(cfg, op, iv.clone(), rc);
				return Value::Temp(result);
			}
		}
		let result;
		let bb_id_to_insert;
		let bb_index_to_insert;

		match (&operand1, &operand2) {
			(Value::Temp(t1), Value::Temp(t2)) => {
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
			(Value::Temp(t), _) | (_, Value::Temp(t)) => {
				(bb_id_to_insert, bb_index_to_insert, _, _) =
					*self.temp_to_instr.get(t).unwrap();
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
				result = self.new_temp(VarType::I32);
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
				result = self.new_temp(VarType::F32);

				let convert_result = self.new_temp(VarType::F32);
				let new_convert_instr = ConvertInstr {
					target: convert_result.clone(),
					op: llvm::ConvertOp::Int2Float,
					from_type: VarType::I32,
					lhs: operand1,
					to_type: VarType::F32,
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
				result = self.new_temp(VarType::F32);

				let convert_result = self.new_temp(VarType::F32);
				let new_convert_instr = ConvertInstr {
					target: convert_result.clone(),
					op: llvm::ConvertOp::Int2Float,
					from_type: VarType::I32,
					lhs: operand2,
					to_type: VarType::F32,
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
				result = self.new_temp(VarType::F32);
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
