// 循指令中 temp 的 use-def 链构造图，每个变量指向它的 use，同时附带上操作名称

use std::collections::HashMap;

use llvm::{ArithOp, LlvmInstr, LlvmInstrVariant, LlvmTemp, Value};
use rrvm::program::LlvmFunc;

use super::loop_data::LoopData;

pub struct Node {
	pub instr: LlvmInstr,
}

pub struct TempGraph {
	pub temp_to_instr: HashMap<LlvmTemp, Node>,
}

impl TempGraph {
	pub fn new() -> Self {
		Self {
			temp_to_instr: HashMap::new(),
		}
	}

	pub fn add_temp(&mut self, temp: LlvmTemp, instr: LlvmInstr) {
		self.temp_to_instr.insert(temp, Node { instr });
	}
	pub fn is_phi(&self, temp: &LlvmTemp) -> bool {
		if let Some(node) = self.temp_to_instr.get(temp) {
			node.instr.is_phi()
		} else {
			false
		}
	}
	pub fn is_candidate_operator(&self, temp: &LlvmTemp) -> Option<ArithOp> {
		if let Some(node) = self.temp_to_instr.get(temp) {
			match node.instr.get_variant() {
				LlvmInstrVariant::ArithInstr(inst) => match inst.op {
					// allow double word indvar
					ArithOp::Add
					| ArithOp::Sub
					| ArithOp::Mul
					| ArithOp::Rem
					| ArithOp::AddD
					| ArithOp::SubD
					| ArithOp::MulD
					| ArithOp::RemD => Some(inst.op),
					_ => None,
				},
				LlvmInstrVariant::GEPInstr(_) => Some(ArithOp::Add),
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

impl LoopData {
	pub fn build_graph(func: &LlvmFunc) -> TempGraph {
		let mut temp_graph = TempGraph::new();
		for bb in func.cfg.blocks.iter() {
			let bb_ = bb.borrow();
			for inst in bb_.phi_instrs.iter() {
				let target = inst.target.clone();
				temp_graph.add_temp(target, Box::new(inst.clone()));
			}
			for inst in bb_.instrs.iter() {
				if let Some(target) = inst.get_write() {
					temp_graph.add_temp(target, inst.clone());
				}
			}
		}
		temp_graph
	}
}
