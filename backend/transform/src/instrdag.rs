use std::{borrow::BorrowMut, cell::RefCell, collections::HashMap, rc::Rc};

use instruction::{riscv::RiscvInstr, temp::TempManager, RiscvInstrSet};
use rrvm::RiscvNode;
use utils::SysycError;

type Node = Rc<RefCell<InstrNode>>;
#[derive(Clone)]
pub struct InstrNode {
	pub in_deg: usize,
	pub instr: RiscvInstr,
	pub succ: Vec<Node>,
	pub last_use: usize,
}
impl InstrNode {
	pub fn new(instr: &RiscvInstr) -> Self {
		Self {
			in_deg: 0,
			instr: instr.clone(),
			succ: Vec::new(),
			last_use: 0,
		}
	}
}
pub struct InstrDag {
	pub nodes: Vec<Node>,
}
impl InstrDag {
	pub fn new(node: &RiscvNode) -> Result<Self, SysycError> {
		let mut nodes: Vec<Node> = Vec::new();
		let mut defs = HashMap::new();
		let mut uses = HashMap::new();
		let mut last_call: Option<Node> = None;
		let mut last_loads: Vec<Node> = Vec::new();
		let mut last_uses = HashMap::new();
		for (idx, instr) in node.borrow().instrs.iter().rev().enumerate() {
			let node = Rc::new(RefCell::new(InstrNode::new(instr)));
			let mut instr_node_succ = Vec::new();
			let instructions_write = instr.get_riscv_write().clone();
			for instr_write in instructions_write.into_iter() {
				instr_node_succ.extend(
					uses.get(&instr_write).unwrap_or(&Vec::new()).iter().cloned(),
				);
				uses.remove(&instr_write);
				defs.insert(instr_write, node.clone());
			}
			let instr_read = instr.get_riscv_read().clone();
			for instr_read_temp in instr_read.iter() {
				if let Some(def_instr) = defs.get(instr_read_temp) {
					instr_node_succ.push(def_instr.clone());
				}
				uses
					.entry(instr_read_temp.clone())
					.or_insert(Vec::new())
					.push(node.clone());
				if !last_uses.contains_key(instr_read_temp) {
					last_uses.insert(instr_read_temp.clone(), idx);
				}
			}
			// 处理 load call store 指令的依赖关系
			if instr.is_call() {
				instr_node_succ.extend(last_loads.iter().cloned());
				last_loads.clear();
				last_call = Some(node.clone());
			} else if instr.is_load().unwrap_or(false) {
				if let Some(last_call) = last_call.clone() {
					instr_node_succ.push(last_call);
				}
				last_loads.push(node.clone());
				last_call = None;
			} else if instr.is_store().unwrap_or(false) {
				instr_node_succ.extend(last_loads.iter().cloned());
				last_loads.clear();
				last_call = Some(node.clone());
			}
			node.borrow_mut().succ = instr_node_succ;
			nodes.push(node);
		}
		Err(SysycError::RiscvGenError("Instrdag::todo".to_string()))
	}
}
