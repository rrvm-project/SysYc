use std::collections::{HashMap, HashSet};

use instruction::riscv::{
	prelude::{BranInstr, RiscvInstrTrait},
	reg::RiscvReg::X0,
	riscvinstr::RiscvInstrVariant::{ITriInstr, RBinInstr, RTriInstr},
	riscvop::{BranInstrOp, IBinInstrOp, ITriInstrOp, RTriInstrOp},
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
pub fn get_xor_writes(
	func: &RiscvFunc,
) -> HashMap<RiscvTemp, (RiscvTemp, RiscvTemp, usize, usize)> {
	let mut xor_writes = HashMap::new();
	for (block_idx, block) in func.cfg.blocks.iter().enumerate() {
		for (instr_idx, instr) in block.borrow().instrs.iter().enumerate() {
			if let RTriInstr(instr) = instr.get_variant() {
				if let RTriInstrOp::Xor = instr.op {
					xor_writes
						.insert(instr.rd, (instr.rs1, instr.rs2, block_idx, instr_idx));
				}
			}
		}
	}
	xor_writes
}
pub fn get_xori_writes(
	func: &RiscvFunc,
) -> HashMap<RiscvTemp, (RiscvTemp, RiscvImm, usize, usize)> {
	let mut xori_writes = HashMap::new();
	for (block_idx, block) in func.cfg.blocks.iter().enumerate() {
		for (instr_idx, instr) in block.borrow().instrs.iter().enumerate() {
			if let ITriInstr(instr) = instr.get_variant() {
				if let ITriInstrOp::Xori = instr.op {
					xori_writes.insert(
						instr.rd,
						(instr.rs1, instr.rs2.clone(), block_idx, instr_idx),
					);
				}
			}
		}
	}
	xori_writes
}
pub fn conditional_branch_combine(func: &mut RiscvFunc) {
	let old_readset = get_readset(func);
	let xor_write = get_xor_writes(func);
	let xori_write = get_xori_writes(func);
	let exceptions = get_new_readset(
		func,
		xor_write.keys().cloned().collect(),
		xori_write.keys().cloned().collect(),
	);
	let readset: HashSet<_> =
		old_readset.iter().filter(|x| !exceptions.contains(x)).cloned().collect();
	// println!("readset:{:?}",readset);
	let branch_reads = get_branch_reads(func, readset.clone());
	// println!("branch_reads:{:?}",branch_reads);
	let (cmpmap, rm_info, replace_info) =
		get_cmpinfo(func, branch_reads, xor_write, xori_write);
	for (idx, block) in func.cfg.blocks.iter_mut().enumerate() {
		branch_combine_block(idx, block, &cmpmap, &rm_info, &replace_info);
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
#[allow(clippy::borrowed_box)]
pub fn filter_func(instr: &Box<dyn RiscvInstrTrait>) -> bool {
	if instr.is_branch() {
		return true;
	}
	if let RBinInstr(_rbin_instr) = instr.get_variant() {
		if instr.get_cmp_op().is_some() {
			return true;
		}
	}
	false
}
pub fn get_new_readset(
	func: &RiscvFunc,
	xor_write: HashSet<RiscvTemp>,
	xori_write: HashSet<RiscvTemp>,
) -> HashSet<RiscvTemp> {
	let mut new_xor_writes: HashSet<_> =
		xor_write.union(&xori_write).copied().collect();
	for block in func.cfg.blocks.iter() {
		for instr in block.borrow().instrs.iter() {
			if !filter_func(instr) {
				for i in instr.get_riscv_read() {
					new_xor_writes.remove(&i);
				}
			}
		}
	}
	new_xor_writes
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
	xor_write: HashMap<RiscvTemp, (RiscvTemp, RiscvTemp, usize, usize)>,
	xori_write: HashMap<RiscvTemp, (RiscvTemp, RiscvImm, usize, usize)>,
) -> (
	HashMap<RiscvTemp, (BranInstrOp, RiscvTemp, RiscvTemp)>,
	HashMap<usize, Vec<usize>>,
	HashMap<(usize, usize), Box<dyn RiscvInstrTrait>>,
) {
	let mut cmpmap = HashMap::new();
	let mut rm_info = HashMap::new();
	let mut replace_info = HashMap::new();
	for (blockidx, block) in func.cfg.blocks.iter().enumerate() {
		for (idx, instr) in block.borrow().instrs.iter().rev().enumerate() {
			if instr.get_riscv_write().iter().any(|x| br_read_regs.contains(x)) {
				if let Some(t) = instr.get_cmp_op() {
					let read_regs = instr.get_riscv_read();
					if read_regs.len() == 2 {
						// get entry of the block idx
						rm_info
							.entry(blockidx)
							.or_insert(vec![])
							.push(block.borrow().instrs.len() - 1 - idx);
						cmpmap.insert(
							instr.get_riscv_write()[0],
							(t, read_regs[0], read_regs[1]),
						);
					}
					// 考虑 seqz,snez 的情况
					else if read_regs.len() == 1 {
						// 先判断是 RBin 还是 ITri
						if let RBinInstr(_rbin_instr) = instr.get_variant() {
							if xor_write.contains_key(&read_regs[0]) {
								let (rs1, rs2, xorblock_idx, xorinstr_idx) =
									xor_write.get(&read_regs[0]).unwrap();
								rm_info
									.entry(*xorblock_idx)
									.or_insert(vec![])
									.push(*xorinstr_idx);
								rm_info
									.entry(blockidx)
									.or_insert(vec![])
									.push(block.borrow().instrs.len() - 1 - idx);
								cmpmap.insert(instr.get_riscv_write()[0], (t, *rs1, *rs2));
							} else if xori_write.contains_key(&read_regs[0]) {
								// 生成新的 li 指令
								let (rs1, rs2, xorblock_idx, xorinstr_idx) =
									xori_write.get(&read_regs[0]).unwrap();
								// 这条指令还是 seqz 之类的
								replace_info.insert(
									(*xorblock_idx, *xorinstr_idx),
									instruction::riscv::riscvinstr::IBinInstr::new(
										IBinInstrOp::Li,
										read_regs[0],
										rs2.clone(),
									),
								);
								cmpmap.insert(read_regs[0], (t, *rs1, read_regs[0]));
								rm_info
									.entry(blockidx)
									.or_insert(vec![])
									.push(block.borrow().instrs.len() - 1 - idx);
							} else {
								rm_info
									.entry(blockidx)
									.or_insert(vec![])
									.push(block.borrow().instrs.len() - 1 - idx);
								cmpmap.insert(
									instr.get_riscv_write()[0],
									(t, read_regs[0], RiscvTemp::PhysReg(X0)),
								);
							}
						} else if let ITriInstr(itri_instr) = instr.get_variant() {
							replace_info.insert(
								(blockidx, block.borrow().instrs.len() - 1 - idx),
								instruction::riscv::riscvinstr::IBinInstr::new(
									IBinInstrOp::Li,
									itri_instr.rd,
									itri_instr.rs2.clone(),
								),
							);
							cmpmap.insert(itri_instr.rd, (t, itri_instr.rs1, itri_instr.rd));
						}
					}
				}
			}
		}
	}
	(cmpmap, rm_info, replace_info)
}
pub fn branch_combine_block(
	cur_idx: usize,
	block: &RiscvNode,
	cmp_map: &HashMap<RiscvTemp, (BranInstrOp, RiscvTemp, RiscvTemp)>,
	rm_info: &HashMap<usize, Vec<usize>>,
	replace_info: &HashMap<(usize, usize), Box<dyn RiscvInstrTrait>>,
) {
	let mut convert_to = None;
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
	}
	// replace info
	// find the instruction to replace
	block.borrow_mut().instrs.iter_mut().enumerate().for_each(|(idx, instr)| {
		if let Some(replace_instr) = replace_info.get(&(cur_idx, idx)) {
			*instr = replace_instr.clone();
		}
	});
	if let Some(mut rm_instrs) = rm_info.get(&cur_idx).cloned() {
		rm_instrs.sort();
		for idx in rm_instrs.iter().rev() {
			block.borrow_mut().instrs.remove(*idx);
		}
	}
}
