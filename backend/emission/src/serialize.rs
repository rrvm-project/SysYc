use std::collections::HashMap;

use instruction::{riscv::riscvinstr::LabelInstr, RiscvInstrSet};
use rrvm::program::RiscvFunc;

pub fn func_serialize(func: RiscvFunc) -> (String, RiscvInstrSet) {
	let mut nodes = func.cfg.blocks;
	let mut pre = HashMap::new();
	nodes.sort_by(|x, y| y.borrow().weight.total_cmp(&x.borrow().weight));
	for node in nodes.iter() {
		let u = node.borrow().id;
		node.borrow_mut().sort_succ();
		if let Some(succ) = node.borrow().succ.first() {
			let v = succ.borrow().id;
			if v != 0 && u != v && pre.get(&v).is_none() {
				pre.insert(v, u);
			}
		}
	}
	nodes.sort_by(|x, y| x.borrow().id.cmp(&y.borrow().id));
	let mut instrs = Vec::new();
	let is_pre = Box::new(|u: i32, v: i32| -> bool {
		pre.get(&v).map_or(false, |v| *v == u)
	});
	for node in nodes {
		if pre.get(&node.borrow().id).is_none() {
			let mut now = node.clone();
			loop {
				instrs.push(LabelInstr::new(now.borrow().label()));
				instrs.append(&mut now.borrow_mut().instrs);
				let v = now.borrow().succ.first().cloned();
				match v {
					Some(succ) if is_pre(now.borrow().id, succ.borrow().id) => now = succ,
					_ => {
						instrs.push(now.borrow_mut().jump_instr.take().unwrap());
						break;
					}
				}
			}
		}
		node.borrow_mut().clear();
	}
	(func.name, instrs)
}
