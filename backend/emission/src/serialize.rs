use std::collections::HashMap;

use instruction::{
	riscv::{
		reg::{RiscvReg::SP, CALLEE_SAVE},
		riscvinstr::{LabelInstr, *},
		riscvop::{
			IBinInstrOp::{LW, SW},
			ITriInstrOp::Addi,
			NoArgInstrOp::Ret,
		},
	},
	RiscvInstrSet,
};
use rrvm::program::RiscvFunc;
use utils::union_find::UnionFind;

pub fn func_serialize(func: RiscvFunc) -> (String, RiscvInstrSet) {
	let size = func.spill_size;
	let mut nodes = func.cfg.blocks;
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
	let mut ret_instrs: RiscvInstrSet = Vec::new();

	instrs.push(ITriInstr::new(Addi, SP.into(), SP.into(), (-96).into()));
	CALLEE_SAVE.iter().skip(1).enumerate().for_each(|(index, &reg)| {
		// TODO: 精确的保存，以及使用寄存器进行 callee-saved
		let instr =
			IBinInstr::new(SW, reg.into(), ((index * 8) as i32, SP.into()).into());
		instrs.push(instr);
		let instr =
			IBinInstr::new(LW, reg.into(), ((index * 8) as i32, SP.into()).into());
		ret_instrs.push(instr);
	});

	instrs.push(ITriInstr::new(Addi, SP.into(), SP.into(), (-size).into()));
	ret_instrs.push(ITriInstr::new(
		Addi,
		SP.into(),
		SP.into(),
		(size + 96).into(),
	));
	ret_instrs.push(NoArgInstr::new(Ret));
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
	// TODO: solve callee saved
	instrs.retain(|v| !v.useless());
	instrs = instrs
		.into_iter()
		.flat_map(|instr| {
			if !instr.is_ret() {
				vec![instr]
			} else {
				ret_instrs.iter().map(|v| v.clone_box()).collect()
			}
		})
		.collect();
	//add func header parse
	let func_header = "  .text\n  .align 2\n  .globl ".to_owned()
		+ &func.name
		+ "\n  .type "
		+ &func.name
		+ ", @function\n"
		+ &func.name;
	(func_header, instrs)
}
