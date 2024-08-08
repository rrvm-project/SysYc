use std::{
	collections::{HashMap, HashSet},
	mem,
};

use instruction::riscv::{
	prelude::{IBinInstr, PCRelLabelInstr, RiscvInstrTrait, RiscvInstrVariant},
	riscvop::IBinInstrOp,
	utils::PCRelMgr,
	value::{RiscvImm, RiscvNumber, RiscvTemp},
};
use rrvm::{program::RiscvFunc, RiscvNode};

pub fn la_reduce_func(
	func: &mut RiscvFunc,
	liveouts: Vec<HashSet<RiscvTemp>>,
	mgr: &mut PCRelMgr,
) {
	for (idx, block) in func.cfg.blocks.iter().enumerate() {
		la_reduce_block(block, &liveouts[idx], mgr, &func.name);
	}
}
pub fn is_ld_store(instr: &dyn RiscvInstrTrait) -> bool {
	if let RiscvInstrVariant::IBinInstr(ibin) = instr.get_variant() {
		if let IBinInstrOp::LD
		| IBinInstrOp::LW
		| IBinInstrOp::LWU
		| IBinInstrOp::FLD
		| IBinInstrOp::FLW
		| IBinInstrOp::FSD
		| IBinInstrOp::FSW
		| IBinInstrOp::SB
		| IBinInstrOp::SD
		| IBinInstrOp::SH
		| IBinInstrOp::SW = ibin.op
		{
			return true;
		}
	}
	false
}
pub fn la_reduce_block(
	block: &RiscvNode,
	liveouts: &HashSet<RiscvTemp>,
	mgr: &mut PCRelMgr,
	func_name: &str,
) {
	// check la instruction and whether the reg has been read
	let mut la_instrs = HashMap::new();
	for (idx, i) in block.borrow().instrs.iter().enumerate() {
		let instr = i.get_variant();
		if let RiscvInstrVariant::IBinInstr(ibin_instr) = instr {
			if ibin_instr.op == IBinInstrOp::LA && !liveouts.contains(&ibin_instr.rd)
			{
				la_instrs.insert(ibin_instr.rd, idx);
			}
		}
	}
	// for i in block instrs if i is not load or store and i reads from la_instrs then remove the instruction from la_instrs
	for instr in block.borrow().instrs.iter() {
		let v = instr.get_riscv_read();
		// check if the instruction reads from la_instrs
		for reg in v.iter() {
			if la_instrs.contains_key(reg) && !is_ld_store(instr.as_ref()) {
				la_instrs.remove(reg);
			}
		}
	}
	if la_instrs.is_empty() {
		return;
	}
	// second iteration: check the offsets of the load and store instructions and whether their offsets are zero
	for instr in block.borrow().instrs.iter() {
		if let RiscvInstrVariant::IBinInstr(ibin) = instr.get_variant() {
			if let IBinInstrOp::LD
			| IBinInstrOp::LW
			| IBinInstrOp::LWU
			| IBinInstrOp::FLD
			| IBinInstrOp::FLW = ibin.op
			{
				// check if the offset is zero and the base register is in la_instrs's keys
				if let RiscvImm::OffsetReg(offset, basereg) = &ibin.rs1 {
					if !(offset.is_zero()) && la_instrs.contains_key(&basereg) {
						la_instrs.remove(basereg);
					}
				}
			}
			if let IBinInstrOp::SB
			| IBinInstrOp::SH
			| IBinInstrOp::SW
			| IBinInstrOp::SD
			| IBinInstrOp::FSW
			| IBinInstrOp::FSD = ibin.op
			{
				// check if the offset is zero and the base register is in la_instrs's keys
				if let RiscvImm::OffsetReg(offset, basereg) = &ibin.rs1 {
					if !(offset.is_zero()) && la_instrs.contains_key(basereg) {
						la_instrs.remove(basereg);
					}
				}
			}
		}
	}
	if la_instrs.is_empty() {
		return;
	}
	eprintln!(" to be replacing las: {:?}", la_instrs);
	// third iteration: transform the la instruction to a lui instruction and replace all the sw and lw's imms
	let instrs = mem::take(&mut block.borrow_mut().instrs);
	block.borrow_mut().instrs = instrs
		.into_iter()
		.enumerate()
		.flat_map(|(_idx, instr)| {
			if let RiscvInstrVariant::IBinInstr(ibin) = instr.get_variant() {
				if let IBinInstrOp::LA = ibin.op {
					if la_instrs.contains_key(&ibin.rd) {
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
				}
				if let IBinInstrOp::LD
				| IBinInstrOp::LW
				| IBinInstrOp::LWU
				| IBinInstrOp::FLD
				| IBinInstrOp::FLW
				| IBinInstrOp::FSD
				| IBinInstrOp::FSW
				| IBinInstrOp::SB
				| IBinInstrOp::SD
				| IBinInstrOp::SH
				| IBinInstrOp::SW = ibin.op
				{
					if let RiscvImm::OffsetReg(_offset, basereg) = &ibin.rs1 {
						if la_instrs.contains_key(basereg) {
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
