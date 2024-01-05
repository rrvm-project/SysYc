use std::{collections::VecDeque, rc::Rc};

use instruction::RiscvInstrSet;
use utils::errors::Result;

use crate::instr_dag::InstrDag;

// TODO: use a better strategy
pub fn instr_schedule(dag: InstrDag) -> Result<RiscvInstrSet> {
	let mut instrs = Vec::new();
	for node in dag.nodes.iter() {
		node.borrow().succ.iter().for_each(|v| v.borrow_mut().in_deg += 1);
	}
	let mut can_alloc: VecDeque<_> =
		dag.nodes.into_iter().filter(|v| v.borrow().in_deg == 0).collect();
	if can_alloc.len() > 1 {
		can_alloc
			.make_contiguous()
			.sort_by(|x, y| y.borrow().last_use.cmp(&x.borrow().last_use))
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
