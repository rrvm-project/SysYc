use std::{collections::VecDeque, rc::Rc};

use instruction::RiscvInstrSet;
use utils::errors::Result;

use crate::instr_dag::InstrDag;

// TODO: use a better strategy
pub fn instr_schedule(dag: InstrDag) -> Result<RiscvInstrSet> {
	let mut can_alloc = VecDeque::new();
	let mut instrs = Vec::new();
	for node in dag.nodes.iter() {
		node.borrow().succ.iter().for_each(|v| v.borrow_mut().in_deg += 1);
	}
	for node in dag.nodes {
		if node.borrow().in_deg == 0 {
			can_alloc.push_back(node);
		}
	}
	while let Some(node) = can_alloc.pop_front() {
		instrs.append(&mut node.borrow_mut().instr);
		for v in node.borrow().succ.iter() {
			v.borrow_mut().in_deg -= 1;
			if v.borrow().in_deg == 0 {
				can_alloc.push_back(Rc::clone(v));
			}
		}
	}
	Ok(instrs)
}
