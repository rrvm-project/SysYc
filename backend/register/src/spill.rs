use std::collections::HashMap;

use instruction::{riscv::prelude::*, temp::*};
use rrvm::program::RiscvFunc;

pub fn spill(
	func: &RiscvFunc,
	map: HashMap<Temp, RiscvImm>,
	mgr: &mut TempManager,
) {
	for node in func.cfg.blocks.iter() {
		let instrs = std::mem::take(&mut node.borrow_mut().instrs);
		node.borrow_mut().instrs = instrs
			.into_iter()
			.flat_map(|mut instr| {
				let mut new_instrs = Vec::new();
				let mut new_map = HashMap::new();
				for temp in instr.get_read() {
					if let Some(addr) = map.get(&temp) {
						let new_temp = mgr.new_raw_temp(&temp, false);
						let load_instr = IBinInstr::new(LD, new_temp.into(), addr.clone());
						new_instrs.push(load_instr);
						new_map.insert(temp, new_temp);
						instr
							.map_src_temp(&[(temp, new_temp.into())].into_iter().collect());
					}
				}
				new_instrs.push(instr.clone());
				if let Some(mut temp) = instr.get_write() {
					if let Some(addr) = map.get(&temp) {
						if let Some(new_temp) = new_map.get(&temp).copied() {
							new_instrs
								.last_mut()
								.unwrap()
								.map_dst_temp(&[(temp, new_temp.into())].into_iter().collect());
							temp = new_temp;
						}
						let store_instr = IBinInstr::new(SD, temp.into(), addr.clone());
						new_instrs.push(store_instr);
					}
				}
				new_instrs
			})
			.collect();
	}
}
