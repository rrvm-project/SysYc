use std::collections::{HashMap, HashSet};

use instruction::{
	riscv::{
		reg::{
			RiscvReg::{SP, X0},
			CALLEE_SAVE,
		},
		riscvinstr::{LabelInstr, *},
		riscvop::{
			BranInstrOp::Beq, IBinInstrOp::*, ITriInstrOp::Addi, NoArgInstrOp::Ret,
		},
		value::RiscvImm,
	},
	RiscvInstrSet,
};
use rrvm::{program::RiscvFunc, RiscvNode};
use utils::{union_find::UnionFind, Label};

pub fn func_serialize(mut nodes: Vec<RiscvNode>) -> RiscvInstrSet {
	let mut pre = HashMap::new();
	let mut union_find = UnionFind::default();
	nodes.sort_by(|x, y| y.borrow().weight.total_cmp(&x.borrow().weight));
	for node in nodes.iter() {
		let u = node.borrow().id;
		node.borrow_mut().sort_succ();
		if let Some(succ) = node.borrow().succ.first() {
			let v = succ.borrow().id;
			if v != 0 && u != v && pre.get(&v).is_none() && !union_find.same(u, v) {
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
	}
	nodes.into_iter().for_each(|v| v.borrow_mut().clear());
	instrs.retain(|v| !v.useless());
	instrs
}

pub fn func_emission(func: RiscvFunc) -> (String, RiscvInstrSet) {
	let mut instrs = func_serialize(func.cfg.blocks);
	let name = func.name;
	let mut prelude = Vec::new();
	let mut exit = vec![LabelInstr::new(Label::new("exit"))];
	let saves: HashSet<_> = instrs
		.iter()
		.flat_map(|v| v.get_riscv_write())
		.filter_map(|v| v.get_phys())
		.filter(|v| CALLEE_SAVE.iter().any(|reg| reg == v))
		.collect();
	let size = ((saves.len() as i32 + func.spills + 1) & !1) * 8;
	if size > 0 {
		prelude.push(ITriInstr::new(Addi, SP.into(), SP.into(), (-size).into()));
	}
	for (index, &reg) in
		CALLEE_SAVE.iter().filter(|v| saves.contains(v)).enumerate()
	{
		let addr: RiscvImm = ((index as i32 + func.spills) * 8, SP.into()).into();
		prelude.push(IBinInstr::new(SD, reg.into(), addr.clone()));
		exit.push(IBinInstr::new(LD, reg.into(), addr));
	}
	if size > 0 {
		exit.push(ITriInstr::new(Addi, SP.into(), SP.into(), (size).into()));
	}
	exit.push(NoArgInstr::new(Ret));
	let exit_addr: RiscvImm = Label::new("exit").into();
	for instr in instrs.iter_mut().filter(|v| v.is_ret()) {
		*instr = BranInstr::new(Beq, X0.into(), X0.into(), exit_addr.clone());
	}
	if let Some(instr) = instrs.last() {
		if instr.get_read_label() == Some(Label::new("exit")) {
			instrs.pop();
		}
	}
	prelude.extend(instrs);
	prelude.extend(exit);
	(name, prelude)
}
