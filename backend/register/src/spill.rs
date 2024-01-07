use instruction::{
	riscv::{riscvinstr::IBinInstr, riscvop::IBinInstrOp::*, value::RiscvImm},
	temp::{Temp, TempManager},
};
use rrvm::program::RiscvFunc;

pub fn spill(
	func: &mut RiscvFunc,
	to_spill: Temp,
	addr: RiscvImm,
	mgr: &mut TempManager,
) {
	let mut flag = true;
	for node in func.cfg.blocks.iter() {
		let instrs = std::mem::take(&mut node.borrow_mut().instrs);
		node.borrow_mut().instrs = instrs
			.into_iter()
			.flat_map(|mut instr| {
				let mut new_instrs = Vec::new();
				let temp = if instr.get_read().into_iter().any(|v| v == to_spill) {
					let new_temp = mgr.new_raw_temp(&to_spill, flag);
					flag = false;
					let load_instr = IBinInstr::new(LD, new_temp.into(), addr.clone());
					new_instrs.push(load_instr);
					instr.map_temp(&[(to_spill, new_temp.into())].into_iter().collect());
					new_temp
				} else {
					to_spill
				};
				new_instrs.push(instr.clone_box());
				match instr.get_write() {
					Some(v) if v == temp => {
						let store_instr = IBinInstr::new(SD, temp.into(), addr.clone());
						new_instrs.push(store_instr);
					}
					_ => {}
				}
				new_instrs
			})
			.collect();
	}
}
