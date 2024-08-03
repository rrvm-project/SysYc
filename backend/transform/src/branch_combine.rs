use std::collections::HashSet;

use instruction::riscv::{
	prelude::BranInstr,
	reg::RiscvReg::X0,
	value::{
		RiscvImm,
		RiscvTemp::{self, PhysReg},
	},
};
use rrvm::{program::RiscvFunc, RiscvNode};

pub fn conditional_branch_combine(
	func: &mut RiscvFunc,
	liveouts: &[HashSet<RiscvTemp>],
) {
	for (idx, block) in func.cfg.blocks.iter_mut().enumerate() {
		branch_combine_block(block, &liveouts[idx]);
	}
}
pub fn branch_combine_block(block: &RiscvNode, liveouts: &HashSet<RiscvTemp>) {
	let mut convert_to = None;
	let mut rm_idx = 0;
	let instr_len = block.borrow().instrs.len();
	if instr_len >= 2 && block.borrow().instrs.last().unwrap().is_branch() {
		// get the read instructions of the last instruction
		let reads = block.borrow().instrs.last().unwrap().get_riscv_read();
		let borrow_block = block.borrow();
		let branch_instr = borrow_block.instrs.last().unwrap();
		if reads.len() == 2 {
			let mut _read_reg = reads[0];
			if let PhysReg(X0) = reads[0] {
				_read_reg = reads[1];
			} else if let PhysReg(X0) = reads[1] {
				_read_reg = reads[0];
			} else {
				return;
			}
			if liveouts.contains(&_read_reg) {
				return;
			}
			for (idx, instr) in block.borrow().instrs.iter().rev().enumerate() {
				if instr.is_branch() {
					continue;
				}
				if instr.get_riscv_write().contains(&_read_reg) {
					if let Some(t) = instr.map_br_op() {
						let read_regs = instr.get_riscv_read();
						if read_regs.len() == 2 {
							rm_idx = idx;
							convert_to = Some(BranInstr::new(
								t,
								read_regs[0],
								read_regs[1],
								RiscvImm::Label(branch_instr.get_read_label().unwrap()),
							));
							// check_reads
							for i in 1..idx {
								if block.borrow().instrs[block.borrow().instrs.len() - 1 - i]
									.get_riscv_read()
									.contains(&_read_reg)
								{
									// 改一下
									return;
								}
							}
						}
					}
					break;
				}
			}
		}
	}
	if let Some(instr) = convert_to {
		block.borrow_mut().instrs.pop();
		block.borrow_mut().instrs.push(instr);
		block.borrow_mut().instrs.remove(instr_len - 1 - rm_idx);
		// eprintln!("Branch combined");
	}
}
