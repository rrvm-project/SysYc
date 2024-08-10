use std::{
	collections::{HashMap, HashSet},
	mem,
};

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
	let tmps = filter_0_offset(func);
	for block in func.cfg.blocks.iter() {
		la_reduce_block(block, &tmps, mgr, &func.name);
	}
}
pub fn filter_0_offset(func: &RiscvFunc) -> HashSet<RiscvTemp> {
	let mut res = HashSet::new();
	for i in func.cfg.blocks.iter() {
		for instr in i.borrow().instrs.iter() {
			if let RiscvInstrVariant::IBinInstr(ibin) = instr.get_variant() {
				if let LD | LW | LWU | FLD | FLW | FSD | FSW | SB | SD | SH | SW =
					ibin.op
				{
					if let RiscvImm::OffsetReg(offset, basereg) = &ibin.rs1 {
						if !offset.is_zero() {
							res.insert(*basereg);
						}
					} else {
						unreachable!();
					}
				} else {
					res.extend(instr.get_riscv_read());
				}
			} else {
				res.extend(instr.get_riscv_read());
			}
		}
	}
	res
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
	if la_instrs.is_empty() {
		return;
	}
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
