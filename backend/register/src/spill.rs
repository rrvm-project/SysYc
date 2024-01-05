use std::collections::HashSet;

use instruction::{
	riscv::{reg::RiscvReg::*, riscvinstr::IBinInstr, riscvop::IBinInstrOp::*},
	temp::{Temp, TempManager},
};
use rrvm::program::RiscvFunc;

pub fn spill(func: &mut RiscvFunc, to_spill: Temp, cnt: i32) {
	// TODO: need more test
	let mut mgr = TempManager::new(cnt);
	let mut stack = vec![(func.cfg.get_entry(), func.spills * 8)];
	let mut visited = HashSet::new();
	let mut flag = true;
	while let Some((node, mut height)) = stack.pop() {
		let id = node.borrow().id;
		visited.insert(id);
		let instrs = std::mem::take(&mut node.borrow_mut().instrs);
		node.borrow_mut().instrs = instrs
			.into_iter()
			.flat_map(|mut instr| {
				instr.move_sp(&mut height);
				let mut new_instrs = Vec::new();
				let temp = if instr.get_read().into_iter().any(|v| v == to_spill) {
					let new_temp = mgr.new_raw_temp(&to_spill, flag);
					flag = false;
					let load_instr =
						IBinInstr::new(LD, new_temp.into(), (height, SP.into()).into());
					new_instrs.push(load_instr);
					instr.map_temp(&[(to_spill, new_temp.into())].into_iter().collect());
					new_temp
				} else {
					to_spill
				};
				new_instrs.push(instr.clone_box());
				match instr.get_write() {
					Some(v) if v == temp => {
						let store_instr =
							IBinInstr::new(SD, temp.into(), (height, SP.into()).into());
						new_instrs.push(store_instr);
					}
					_ => {}
				}
				new_instrs
			})
			.collect();
		for v in node.borrow().succ.iter() {
			if visited.get(&v.borrow().id).is_none() {
				stack.push((v.clone(), height))
			}
		}
	}
	func.spills += 1;
}
