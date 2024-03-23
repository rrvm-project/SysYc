use std::collections::{HashMap, HashSet};

use instruction::{riscv::prelude::*, RiscvInstrSet};
use rrvm::{program::RiscvFunc, RiscvNode};
use utils::{math::align16, union_find::UnionFind, Label};

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
	let name = func.name;
	instrs = solve_caller_save(instrs);
	solve_callee_save(&mut instrs, func.spills);
	instrs.retain(|v| !v.useless());
	(name, instrs)
}

fn solve_callee_save(instrs: &mut RiscvInstrSet, spills: i32) {
	let mut prelude = Vec::new();
	let mut epilogue = vec![LabelInstr::new(Label::new("exit"))];
	let mut saves: HashSet<_> = instrs
		.iter()
		.flat_map(|v| v.get_riscv_write())
		.filter_map(|v| v.get_phys())
		.filter(|v| CALLEE_SAVE.iter().any(|reg| reg == v))
		.collect();
	if instrs.iter().any(|instr| {
		instr.get_riscv_read().iter().any(|v| v.get_phys().is_some_and(|v| v == FP))
	}) {
		saves.insert(FP);
	}
	let size = ((saves.len() as i32 + spills + 1) * 8) & -16;
	if size > 0 {
		prelude.push(ITriInstr::new(Addi, SP.into(), SP.into(), (-size).into()));
	}
	for (index, &reg) in
		CALLEE_SAVE.iter().filter(|v| saves.contains(v)).enumerate()
	{
		let addr: RiscvImm = (index as i32 * 8, SP.into()).into();
		prelude.push(IBinInstr::new(SD, reg.into(), addr.clone()));
		epilogue.push(IBinInstr::new(LD, reg.into(), addr));
	}
	if size > 0 {
		if saves.contains(&FP) {
			prelude.push(ITriInstr::new(Addi, FP.into(), SP.into(), size.into()));
		}
		epilogue.push(ITriInstr::new(Addi, SP.into(), SP.into(), size.into()));
	}
	epilogue.push(NoArgInstr::new(Ret));
	let epilogue_addr: RiscvImm = Label::new("exit").into();
	for instr in instrs.iter_mut().filter(|v| v.is_ret()) {
		*instr = BranInstr::new(Beq, X0.into(), X0.into(), epilogue_addr.clone());
	}
	if let Some(instr) = instrs.last() {
		if instr.get_read_label() == Some(Label::new("exit")) {
			instrs.pop();
		}
	}
	prelude.append(instrs);
	prelude.extend(epilogue);
	*instrs = prelude;
}

fn solve_caller_save(instrs: RiscvInstrSet) -> RiscvInstrSet {
	let saves: HashSet<_> = instrs
		.iter()
		.flat_map(|v| v.get_riscv_read())
		.filter_map(|v| v.get_phys())
		.filter(|v| CALLER_SAVE.iter().skip(1).any(|reg| reg == v))
		.collect();
	let size = align16(saves.len() as i32 * 8);
	if size > 0 {
		let mut prelude = Vec::new();
		let mut epilogue = Vec::new();
		prelude.push(ITriInstr::new(Addi, SP.into(), SP.into(), (-size).into()));
		saves.into_iter().enumerate().for_each(|(index, v)| {
			let offset = (index * 8) as i32;
			prelude.push(IBinInstr::new(SD, v.into(), (offset, SP.into()).into()));
			epilogue.push(IBinInstr::new(LD, v.into(), (offset, SP.into()).into()));
		});
		epilogue.push(ITriInstr::new(Addi, SP.into(), SP.into(), size.into()));
		instrs
			.into_iter()
			.flat_map(|instr| match instr.get_temp_op() {
				Some(Save) => prelude.iter().map(|v| v.clone_box()).collect(),
				Some(Restore) => epilogue.iter().map(|v| v.clone_box()).collect(),
				None => vec![instr],
			})
			.collect()
	} else {
		instrs.into_iter().filter(|instr| instr.get_temp_op().is_none()).collect()
	}
}
