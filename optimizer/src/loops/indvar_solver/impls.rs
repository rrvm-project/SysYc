// 寻找归纳变量的算法
use std::collections::{HashMap, HashSet};

use llvm::{ArithOp, LlvmTemp, LlvmTempManager, Value};
use rrvm::{rrvm_loop::LoopPtr, LlvmCFG, LlvmNode};

use crate::{
	loops::{chain_node::ChainNode, indvar::IndVar, temp_graph::TempGraph},
	metadata::FuncData,
};

use super::IndVarSolver;

impl<'a> IndVarSolver<'a> {
	#[allow(clippy::too_many_arguments)]
	pub fn new(
		cfg: &'a mut LlvmCFG,
		params: HashSet<LlvmTemp>,
		cur_loop: LoopPtr,
		preheader: LlvmNode,
		mgr: &'a mut LlvmTempManager,
		temp_graph: &'a mut TempGraph,
		loop_map: &'a mut HashMap<i32, LoopPtr>,
		def_map: &'a mut HashMap<LlvmTemp, LlvmNode>,
		funcdata: &'a mut FuncData,
	) -> Self {
		Self {
			dfsnum: HashMap::new(),
			next_dfsnum: 0,
			visited: HashSet::new(),
			low: HashMap::new(),
			stack: Vec::new(),
			in_stack: HashSet::new(),
			loop_invariant: HashSet::new(),
			header_map: HashMap::new(),
			cur_loop,
			preheader,
			variants: HashSet::new(),
			useful_variants: HashSet::new(),
			useless_variants: HashSet::new(),
			indvars: HashMap::new(),
			params,
			new_invariant_instr: HashMap::new(),
			flag: false,
			temp_graph,
			mgr,
			loop_map,
			def_map,
			funcdata,
			cfg,
		}
	}
	pub fn run(&mut self, start: LlvmTemp) {
		self.tarjan(start);
		// self.lstf(cfg, mgr);
	}
	pub fn tarjan(&mut self, temp: LlvmTemp) {
		self.visited.insert(temp.clone());
		self.dfsnum.insert(temp.clone(), self.next_dfsnum);
		self.low.insert(temp.clone(), self.next_dfsnum);
		self.next_dfsnum += 1;
		self.stack_push(temp.clone());
		let mut reads: Vec<LlvmTemp> = self.temp_graph.get_use_temps(&temp);
		// 只保留在当前循环中的变量
		reads.retain(|t| self.is_temp_in_current_loop(t));

		reads.iter().for_each(|operand| {
			if !self.visited.contains(operand) {
				self.tarjan(operand.clone());
				self.low.insert(temp.clone(), self.low[&temp].min(self.low[operand]));
			} else if self.dfsnum[operand] < self.dfsnum[&temp] // TODO: Tarjan 算法中 这里需不需要判断 dfsnum 的大小，我感觉不用
				&& self.stack_contains(operand)
			{
				self
					.low
					.insert(temp.clone(), self.low[&temp].min(self.dfsnum[operand]));
			}
		});
		if self.dfsnum[&temp] == self.low[&temp] {
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
			// let member = &scc[0];
			// let (_, bb_id, instr_id, is_phi) =
			// 	self.temp_to_instr.get(member).unwrap();
			// if let Some((iv, rc)) =
			// 	self.is_candidate_operation(cfg, *bb_id, *instr_id, *is_phi)
			// {
			// 	self.replace(cfg, member.clone(), iv, rc, mgr);
			// } else {
			// 	self.header.remove(member);
			// }
			let member = scc[0].clone();
			if self.classify_single_member_scc(&member) {
				self.variants.insert(member.clone());
			} else {
				eprintln!("IndVarSolver: found a loop invariant {}", member);
				// 是循环不变量，记录下来，之后把它移到 preheader 中
				self.loop_invariant.insert(member.clone());
			}
		} else {
			// 获得 scc 在 header 中的 phi 语句的 target
			let header = self.find_header_for_scc(&scc);
			scc.iter().for_each(|t| {
				self.header_map.insert(t.clone(), header.clone());
			});
			self.variants.insert(header.clone());
			self.classify_induction_variables(header);
		}
	}
	pub fn classify_induction_variables(&mut self, header: LlvmTemp) {
		let mut is_zfp = None;
		let reads = self.temp_graph.get_use_values(&header);
		assert!(reads.len() == 2);
		let member1 = reads[0].clone();
		let member2 = reads[1].clone();
		let (variant, phi_base) = self.get_variant_and_step(&member1, &member2, &header).expect("header of a scc must have a operand of indvar and a operand of a invariant");
		let mut indvar_chain: Vec<ChainNode> = vec![];

		let mut chain_runner = variant;
		// 允许最后一个操作是 mod. 由于我这里是逆向访问 chain, 所以需要检查第一个被访问的操作是不是 mod
		let reads = self.temp_graph.get_use_values(&chain_runner);
		if self.temp_graph.is_mod(&chain_runner) {
			if let Some((variant, step)) =
				self.get_variant_and_step(&reads[0], &reads[1], &header)
			{
				assert!(self.header_map[&variant] == header);
				chain_runner = variant.clone();
				indvar_chain.push(ChainNode::new(variant, ArithOp::Rem, step.clone()));
				is_zfp = Some(step);
			} else {
				return;
			}
		}
		while chain_runner != header {
			let reads = self.temp_graph.get_use_values(&chain_runner);
			// For it to be on a chain, it must have at least one read
			assert!(!reads.is_empty());
			// TODO: 目前只允许 chain 中的操作是整数加法，减法，乘法
			if let Some(chain_op) =
				self.temp_graph.is_candidate_operator(&chain_runner)
			{
				if let Some((variant, step)) =
					self.get_variant_and_step(&reads[0], &reads[1], &header)
				{
					assert!(self.header_map[&variant] == header);
					chain_runner = variant.clone();
					indvar_chain.push(ChainNode::new(variant, chain_op, step))
				} else {
					return;
				}
			} else {
				return;
			}
		}
		self.compute_indvar(indvar_chain, header, phi_base, is_zfp);
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
		let mut step = Value::Int(0);
		let mut scale = Value::Int(1);
		let mut base = phi_base.clone();
		let mut indvar_bases = vec![];
		for indvar in indvar_chain[..end].iter() {
			match indvar.op {
				ArithOp::Add => {
					step =
						self.compute_two_value(step, indvar.operand.clone(), ArithOp::Add);
					base =
						self.compute_two_value(base, indvar.operand.clone(), ArithOp::Add);
					indvar_bases.push(base.clone());
				}
				ArithOp::Sub => {
					step =
						self.compute_two_value(step, indvar.operand.clone(), ArithOp::Sub);
					base =
						self.compute_two_value(base, indvar.operand.clone(), ArithOp::Sub);
					indvar_bases.push(base.clone());
				}
				ArithOp::Mul => {
					scale =
						self.compute_two_value(scale, indvar.operand.clone(), ArithOp::Mul);
					base =
						self.compute_two_value(base, indvar.operand.clone(), ArithOp::Mul);
					step =
						self.compute_two_value(step, indvar.operand.clone(), ArithOp::Mul);
					indvar_bases.push(base.clone());
				}
				_ => unreachable!(),
			}
		}
		eprintln!(
			"IndVarSolver: found a indvar {} with base: {}, scale: {}, step: {}",
			header, phi_base, scale, step
		);
		self.indvars.insert(
			header.clone(),
			IndVar::new(phi_base, scale.clone(), step.clone(), is_zfp.clone()),
		);
		for (indvar, indvar_base) in indvar_chain.iter().zip(indvar_bases) {
			self.indvars.insert(
				indvar.temp.clone(),
				IndVar::new(indvar_base, scale.clone(), step.clone(), is_zfp.clone()),
			);
		}
	}
}
