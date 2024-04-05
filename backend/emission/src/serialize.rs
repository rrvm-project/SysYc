use std::collections::{HashMap, HashSet};

use instruction::{riscv::prelude::*, RiscvInstrSet};
use rrvm::{program::RiscvFunc, RiscvNode};
use utils::union_find::UnionFind;

fn func_serialize(mut nodes: Vec<RiscvNode>) -> RiscvInstrSet {
	let mut pre = HashMap::new();
	let mut union_find = UnionFind::default();
	nodes.sort_by(|x, y| y.borrow().weight.total_cmp(&x.borrow().weight));
	for node in nodes.iter() {
		let u = node.borrow().id;
		node.borrow_mut().sort_succ();
		if let Some(succ) = node.borrow().succ.first() {
			let v = succ.borrow().id;
			if v != 0 && u != v && !pre.contains_key(&v) && !union_find.same(u, v) {
				pre.insert(v, u);
				union_find.merge(u, v);
			}
		}
	}
	nodes.sort_by(|x, y| x.borrow().id.cmp(&y.borrow().id));
	let mut instrs = Vec::new();
	let is_pre = Box::new(|u: i32, v: i32| -> bool {
		pre.get(&v).map_or(false, |v| *v == u)
	});
	for node in nodes.iter() {
		if !pre.contains_key(&node.borrow().id) {
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
	}
	nodes.into_iter().for_each(|v| v.borrow_mut().clear());
	instrs
}

pub fn func_emission(func: RiscvFunc) -> (String, RiscvInstrSet) {
	let mut instrs = func_serialize(func.cfg.blocks);
	instrs.retain(|v| !v.useless());
	let labels: HashSet<_> =
		instrs.iter().filter_map(|v| v.get_read_label()).collect();
	instrs.retain(|v| v.get_write_label().map_or(true, |v| labels.contains(&v)));
	(func.name, instrs)
}
