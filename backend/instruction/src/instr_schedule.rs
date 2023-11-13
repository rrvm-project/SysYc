use std::rc::Rc;

use utils::SysycError;

use crate::{instr_dag::InstrDag, InstrSet};

// TODO: construct InstrSet
pub fn instr_serialize(dag: InstrDag) -> Result<InstrSet, SysycError> {
	let mut can_alloc = Vec::new();
	// let mut instrs = Vec::new();
	for node in dag.nodes {
		if node.borrow().in_deg == 0 {
			can_alloc.push(node);
		}
	}
	while let Some(node) = can_alloc.pop() {
		for v in node.borrow().succ.iter() {
			v.borrow_mut().in_deg -= 1;
			if v.borrow().in_deg == 0 {
				can_alloc.push(Rc::clone(v));
			}
		}
	}
	todo!()
	// InstrSet::RiscvInstrSet()
}
