use std::{collections::HashSet, mem};

use instruction::riscv::{
	prelude::{IBinInstr, PCRelLabelInstr, RiscvInstrVariant},
	riscvop::{IBinInstrOp, IBinInstrOp::*},
	utils::PCRelMgr,
	value::{RiscvImm, RiscvNumber, RiscvTemp},
};
use rrvm::{
	program::{RiscvFunc, RiscvProgram},
	RiscvNode,
};

pub fn la_reduce(program: &mut RiscvProgram) {
	let mut mgr = PCRelMgr::default();
	for func in program.funcs.iter_mut() {
		la_reduce_func(func, &mut mgr);
	}
}
pub fn la_reduce_func(func: &mut RiscvFunc, mgr: &mut PCRelMgr) {
	let tmps = filter_zero_offset(func);
	let la_writes = find_la_writes(func, &tmps);
	if la_writes.is_empty() {
		return;
	}
	for block in func.cfg.blocks.iter() {
		la_reduce_block(block, mgr, &func.name, &la_writes);
	}
}
pub fn filter_zero_offset(func: &RiscvFunc) -> HashSet<RiscvTemp> {
	let mut res = HashSet::new();
	for i in func.cfg.blocks.iter() {
		for instr in i.borrow().instrs.iter() {
			if let RiscvInstrVariant::IBinInstr(ibin) = instr.get_variant() {
				if let RiscvImm::OffsetReg(RiscvNumber::Int(0), basereg) = &ibin.rs1 {
					res.insert(*basereg);
				}
			} else {
				res.extend(instr.get_riscv_read());
			}
		}
	}
	res
}
pub fn find_la_writes(
	func: &RiscvFunc,
	liveouts: &HashSet<RiscvTemp>,
) -> HashSet<RiscvTemp> {
	let mut res = HashSet::new();
	for i in func.cfg.blocks.iter() {
		for instr in i.borrow().instrs.iter() {
			if let RiscvInstrVariant::IBinInstr(ibin) = instr.get_variant() {
				if LA == ibin.op && !liveouts.contains(&ibin.rd) {
					res.insert(ibin.rd);
				}
			}
		}
	}
	res
}
pub fn la_reduce_block(
	block: &RiscvNode,
	mgr: &mut PCRelMgr,
	func_name: &str,
	la_writes: &HashSet<RiscvTemp>,
) {
	// third iteration: transform the la instruction to a lui instruction and replace all the sw and lw's imms
	let instrs = mem::take(&mut block.borrow_mut().instrs);
	block.borrow_mut().instrs = instrs
		.into_iter()
		.enumerate()
		.flat_map(|(_idx, instr)| {
			if let RiscvInstrVariant::IBinInstr(ibin) = instr.get_variant() {
				if LA == ibin.op && la_writes.contains(&ibin.rd) {
					if let RiscvImm::Label(label) = &ibin.rs1 {
						let pcrel_label_instr = PCRelLabelInstr::new(
							mgr.get_new_label(func_name.to_string(), ibin.rd),
						);
						let auipc_instr = IBinInstr::new(
							IBinInstrOp::Auipc,
							ibin.rd,
							RiscvImm::RiscvNumber(RiscvNumber::Hi(label.clone())),
						);
						return vec![pcrel_label_instr, auipc_instr];
					}
				}

				if let LD | LW | LWU | FLD | FLW | FSD | FSW | SB | SD | SH | SW =
					ibin.op
				{
					if let RiscvImm::OffsetReg(_offset, basereg) = &ibin.rs1 {
						if la_writes.contains(basereg) {
							let new_offset = RiscvImm::OffsetReg(
								RiscvNumber::Lo(utils::Label {
									name: mgr.find_label(func_name, basereg).unwrap().to_string(),
								}),
								*basereg,
							);
							let new_instr = IBinInstr::new(ibin.op, ibin.rd, new_offset);
							return vec![new_instr];
						}
					}
				}
			}
			vec![instr]
		})
		.collect();
}
