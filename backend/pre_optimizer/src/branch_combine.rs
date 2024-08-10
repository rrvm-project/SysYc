use std::collections::{HashMap, HashSet};

use instruction::riscv::{
	prelude::BranInstr,
	reg::RiscvReg::X0,
	riscvop::BranInstrOp,
	value::{
		RiscvImm,
		RiscvTemp::{self, PhysReg},
	},
};
use rrvm::{
	program::{RiscvFunc, RiscvProgram},
	RiscvNode,
};

pub fn branch_combine(program: &mut RiscvProgram) {
	for func in program.funcs.iter_mut() {
		conditional_branch_combine(func);
	}
}

pub fn conditional_branch_combine(func: &mut RiscvFunc) {
	let readset = get_readset(func);
	let branch_reads = get_branch_reads(func, readset.clone());
	let (cmpmap, rm_info) = get_cmpinfo(func, branch_reads);
	for (idx, block) in func.cfg.blocks.iter_mut().enumerate() {
		branch_combine_block(idx, block, &cmpmap, &rm_info);
	}
}
pub fn get_readset(func: &RiscvFunc) -> HashSet<RiscvTemp> {
	let mut readset = HashSet::new();
	for block in func.cfg.blocks.iter() {
		for instr in block.borrow().instrs.iter() {
			if !instr.is_branch() {
				for reg in instr.get_riscv_read() {
					readset.insert(reg);
				}
			}
		}
	}
	readset
}
pub fn get_branch_reads(
	func: &RiscvFunc,
	read_regs: HashSet<RiscvTemp>,
) -> HashSet<RiscvTemp> {
	let mut branch_reads = HashSet::new();
	for block in func.cfg.blocks.iter() {
		for instr in block.borrow().instrs.iter() {
			if instr.is_branch() {
				for reg in instr.get_riscv_read() {
					if !read_regs.contains(&reg) {
						branch_reads.insert(reg);
					}
				}
			}
		}
	}
	branch_reads
}
#[allow(clippy::type_complexity)]
pub fn get_cmpinfo(
	func: &RiscvFunc,
	br_read_regs: HashSet<RiscvTemp>,
) -> (
	HashMap<RiscvTemp, (BranInstrOp, RiscvTemp, RiscvTemp)>,
	HashMap<RiscvTemp, (usize, usize)>,
) {
	let mut cmpmap = HashMap::new();
	let mut rm_info = HashMap::new();
	for (blockidx, block) in func.cfg.blocks.iter().enumerate() {
		for (idx, instr) in block.borrow().instrs.iter().rev().enumerate() {
			if instr.get_riscv_write().iter().any(|x| br_read_regs.contains(x)) {
				if let Some(t) = instr.get_cmp_op() {
					let read_regs = instr.get_riscv_read();
					if read_regs.len() == 2 {
						rm_info.insert(
							instr.get_riscv_write()[0],
							(blockidx, block.borrow().instrs.len() - 1 - idx),
						);
						cmpmap.insert(
							instr.get_riscv_write()[0],
							(t, read_regs[0], read_regs[1]),
						);
					}
				}
			}
		}
	}
	(cmpmap, rm_info)
}
pub fn branch_combine_block(
	cur_idx: usize,
	block: &RiscvNode,
	cmp_map: &HashMap<RiscvTemp, (BranInstrOp, RiscvTemp, RiscvTemp)>,
	rm_info: &HashMap<RiscvTemp, (usize, usize)>,
) {
	let mut convert_to = None;
	let mut rm_idx = None;
	let instr_len = block.borrow().instrs.len();
	if instr_len >= 1 {
		let last_instr = block.borrow_mut().instrs.pop().unwrap();
		// get the read instructions of the last instruction
		if last_instr.is_branch() {
			let reads = last_instr.get_riscv_read();
			if reads.len() == 2 {
				let mut _read_reg = reads[0];
				if let PhysReg(X0) = reads[0] {
					_read_reg = reads[1];
				} else if let PhysReg(X0) = reads[1] {
					_read_reg = reads[0];
				} else {
					return;
				}
				if cmp_map.contains_key(&_read_reg) {
					convert_to = cmp_map.get(&_read_reg).cloned();
				}
				if let Some((block_idx, idx)) = rm_info.get(&_read_reg) {
					if *block_idx == cur_idx {
						rm_idx = Some(*idx);
					}
				}
			}
		}
		if let Some(instr) = convert_to {
			block.borrow_mut().instrs.push(BranInstr::new(
				instr.0,
				instr.1,
				instr.2,
				RiscvImm::Label(last_instr.get_read_label().unwrap()),
			));
		} else {
			block.borrow_mut().instrs.push(last_instr);
		}
		if let Some(idx) = rm_idx {
			block.borrow_mut().instrs.remove(idx);
		}
	}
}
