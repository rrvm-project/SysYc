use std::collections::{HashMap, HashSet};

use instruction::{
	riscv::{prelude::*, virt_mem::VirtMemManager},
	temp::*,
};
use rrvm::program::RiscvFunc;

pub fn spill(
	func: &RiscvFunc,
	nodes: &HashSet<Temp>,
	mgr: &mut TempManager,
	mem_mgr: &mut VirtMemManager,
) {
	for node in func.cfg.blocks.iter() {
		let instrs = std::mem::take(&mut node.borrow_mut().instrs);
		node.borrow_mut().instrs = instrs
			.into_iter()
			.flat_map(|mut instr| {
				let mut new_instrs = Vec::new();
				let mut new_map = HashMap::new();
				for temp in instr.get_read() {
					if nodes.contains(&temp) {
						let addr = mem_mgr.get_mem(temp.into());
						let new_temp = mgr.new_raw_temp(&temp, false, temp.var_type);
						let load_instr = IBinInstr::new(LD, new_temp.into(), addr.into());
						new_instrs.push(load_instr);
						new_map.insert(temp, new_temp);
						instr
							.map_src_temp(&[(temp, new_temp.into())].into_iter().collect());
					}
				}
				new_instrs.push(instr.clone());
				if let Some(mut temp) = instr.get_write() {
					if nodes.contains(&temp) {
						let addr = mem_mgr.get_mem(temp.into());
						if let Some(new_temp) = new_map.get(&temp).copied() {
							new_instrs
								.last_mut()
								.unwrap()
								.map_dst_temp(&[(temp, new_temp.into())].into_iter().collect());
							temp = new_temp;
						}
						let store_instr = IBinInstr::new(SD, temp.into(), addr.into());
						new_instrs.push(store_instr);
					}
				}
				new_instrs
			})
			.collect();
	}
}
