// Ref：Engineering a Compiler 2nd Edition Page 433
mod helper_functions;
use std::{
	collections::{HashMap, HashSet, VecDeque},
	rc,
};

use llvm::{ArithInstr, ArithOp, HashableValue, LlvmInstr, Temp, Value};
use rrvm::{dominator::naive::compute_dominator, LlvmCFG, LlvmNode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InductionVariableState {
	Valid,
	InValid,
	Unknown,
}

pub struct OSR {
	// dfs 过程中，访问到的次序
	dfsnum: HashMap<Temp, i32>,
	next_dfsnum: i32,
	visited: HashMap<Temp, bool>,
	// Tarjan 算法计算强连通分量时，需要用到的值
	low: HashMap<Temp, i32>,
	stack: Vec<Temp>,
	header: HashMap<Temp, Temp>,
	// 临时变量到（基本块id，基本块数组下标，指令数组下标）的映射
	temp_to_instr: HashMap<Temp, (i32, usize, usize)>,

	// 记录因为候选操作而产生的指令，防止产生重复的指令
	new_instr: HashMap<(ArithOp, HashableValue, HashableValue), Temp>,
	total_new_temp: u32,
	// 此过程是否做出了优化
	flag: bool,

	dominates: HashMap<i32, Vec<LlvmNode>>,
	dominates_directly: HashMap<i32, Vec<LlvmNode>>,
	dominator: HashMap<i32, LlvmNode>,
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
					temp_to_instr.insert(temp.clone(), (block.id, bb_id, instr_id));
				});
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
			dominates_directly,
			dominator,
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
		let (_, bb_id, instr_id) = self.temp_to_instr.get(&temp).unwrap();
		let reads = &cfg.blocks[*bb_id].borrow().instrs[*instr_id].get_read();
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
			let (_, bb_id, instr_id) = self.temp_to_instr.get(member).unwrap();
			if let Some((iv, rc)) =
				self.is_candidate_operation(cfg, *bb_id, *instr_id)
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
			let (_, bb_id, instr_id) = self.temp_to_instr.get(&p).unwrap();
			let instr = &cfg.blocks[*bb_id].borrow().instrs[*instr_id];
			if instr.is_phi() {
				visited.insert(p.clone(), InductionVariableState::Valid);
			} else if instr.get_read().iter().any(|operand| {
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
						if let Some((iv, header)) = self.is_induction_value(lhs.clone()) {
							if let Some(rc) = self.is_regional_constant(header.clone(), rhs) {
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
							if let Some(rc) =
								self.is_regional_constant(scc_header.clone(), rhs)
							{
								visited.insert(p.clone(), InductionVariableState::Valid);
							} else {
								is_induction_variable = false;
								break;
							}
						} else if let Some((iv, header)) =
							self.is_induction_value(rhs.clone())
						{
							if let Some(rc) = self.is_regional_constant(header.clone(), lhs) {
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
							if let Some(rc) =
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
						if let Some((iv, header)) = self.is_induction_value(lhs.clone()) {
							if let Some(rc) = self.is_regional_constant(header.clone(), rhs) {
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
							if let Some(rc) =
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
				let (_, bb_id, instr_id) = self.temp_to_instr.get(p).unwrap();
				if let Some((iv, rc)) =
					self.is_candidate_operation(cfg, *bb_id, *instr_id)
				{
					self.replace(cfg, p.clone(), iv, rc);
				} else {
					self.header.remove(p);
				}
			}
		}
	}
	// 对候选操作进行替换
	pub fn replace(
		&mut self,
		cfg: &mut LlvmCFG,
		scc_member: Temp,
		iv: Temp,
		rc: Value,
	) {
		let (_, bb_id, instr_id) =
			*self.temp_to_instr.get(&scc_member).unwrap();
		let op = cfg.blocks[bb_id].borrow().instrs[instr_id]
			.is_candidate_operator()
			.unwrap();
		let result = self.reduce(cfg, op, iv.clone(), rc);
		self.replace_to_copy(cfg, bb_id, instr_id, result);
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
		let result =  self.new_temp(&iv);
		self.new_instr.insert(
			(
				op,
				HashableValue::from(Value::Temp(iv.clone())),
				HashableValue::from(rc.clone()),
			),
			result.clone(),
		);
		let (id, bb_id, instr_id) = *self.temp_to_instr.get(&iv).unwrap();
		let instr = &cfg.blocks[bb_id].borrow().instrs[instr_id];
		let mut new_def = instr.clone_box();
		new_def.swap_target(result.clone());

		cfg.blocks[bb_id].borrow_mut().instrs[((instr_id)+1)..].iter().for_each(|i| i.get_write().iter().for_each(|t| {
			self.temp_to_instr.entry(t.clone()).and_modify(|(_, _, instr_id)| {
				*instr_id = *instr_id + 1;
			});
		}));
		cfg.blocks[bb_id].borrow_mut().instrs.insert(instr_id + 1, new_def);

		self.temp_to_instr.insert(result.clone(), (id, bb_id, instr_id + 1));
		self.dfsnum.insert(result.clone(), self.dfsnum[&iv]);
		self.visited.insert(result.clone(), self.visited[&iv]);
		self.low.insert(result.clone(), self.low[&iv]);
		self.header.insert(result.clone(), self.header[&iv].clone());

		let new_def = &cfg.blocks[bb_id].borrow().instrs[instr_id + 1];
		todo!()
	}
	pub fn apply(
		&mut self,
		cfg: &mut LlvmCFG,
		op: ArithOp,
		operand1: Value,
		operand2: Value,
	) -> Value {
		todo!()
	}
}
