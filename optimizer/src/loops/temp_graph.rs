// 循指令中 temp 的 use-def 链构造图，每个变量指向它的 use，同时附带上操作名称

use std::collections::HashMap;

use llvm::{ArithOp, LlvmInstr, LlvmInstrVariant, LlvmTemp, Value};
use rrvm::program::LlvmFunc;

use super::loop_optimizer::LoopOptimizer;

pub struct Node {
	pub instr: LlvmInstr,
}

pub struct TempGraph {
	pub temp_to_instr: HashMap<LlvmTemp, Node>,
	// pub temp_to_optype: HashMap<LlvmTemp, OpType>,
	// // 从自己指向自己的 use
	// pub temp_graph: HashMap<LlvmTemp, HashSet<Value>>,
}

#[allow(unused)]
impl TempGraph {
	pub fn new() -> Self {
		Self {
			temp_to_instr: HashMap::new(),
		}
	}

	pub fn add_temp(&mut self, temp: LlvmTemp, instr: LlvmInstr) {
		self.temp_to_instr.insert(temp, Node { instr });
	}

	// load 非全局变量的 load 才是 load
	pub fn is_load(&self, temp: &LlvmTemp) -> bool {
		if let Some(node) = self.temp_to_instr.get(temp) {
			node.instr.is_load()
		} else {
			println!("temp: {:?} not found", temp);
			false
		}
	}
	pub fn is_phi(&self, temp: &LlvmTemp) -> bool {
		if let Some(node) = self.temp_to_instr.get(temp) {
			node.instr.is_phi()
		} else {
			false
		}
	}
	pub fn is_call(&self, temp: &LlvmTemp) -> bool {
		if let Some(node) = self.temp_to_instr.get(temp) {
			node.instr.is_call()
		} else {
			false
		}
	}
	pub fn is_candidate_operator(&self, temp: &LlvmTemp) -> Option<ArithOp> {
		if let Some(node) = self.temp_to_instr.get(temp) {
			match node.instr.get_variant() {
				LlvmInstrVariant::ArithInstr(inst) => match inst.op {
					ArithOp::Add | ArithOp::Sub | ArithOp::Mul | ArithOp::Rem => {
						Some(inst.op)
					}
					_ => None,
				},
				_ => None,
			}
		} else {
			None
		}
	}
	pub fn is_mod(&self, temp: &LlvmTemp) -> bool {
		self.is_candidate_operator(temp).is_some_and(|op| op == ArithOp::Rem)
	}
	pub fn get_use_temps(&self, temp: &LlvmTemp) -> Vec<LlvmTemp> {
		self.temp_to_instr[temp].instr.get_read()
	}
	pub fn get_use_values(&self, temp: &LlvmTemp) -> Vec<Value> {
		self.temp_to_instr[temp].instr.get_read_values()
	}
}

impl<'a> LoopOptimizer<'a> {
	pub fn build_graph(func: &LlvmFunc) -> TempGraph {
		let mut temp_graph = TempGraph::new();
		for bb in func.cfg.blocks.iter() {
			for inst in bb.borrow().phi_instrs.iter() {
				let target = inst.target.clone();
				temp_graph.add_temp(target, Box::new(inst.clone()));
			}
			for inst in bb.borrow().instrs.iter() {
				if let Some(target) = inst.get_write() {
					temp_graph.add_temp(target, inst.clone());
				}
			}
		}
		temp_graph
	}
}
