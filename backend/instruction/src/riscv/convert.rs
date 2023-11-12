#![allow(unused)]

use llvm::{llvminstr::*, temp::TempManager};

use crate::InstrSet;

pub fn riscv_arith(instr: &ArithInstr, mgr: &mut TempManager) -> InstrSet {
	todo!()
}

pub fn riscv_label(instr: &LabelInstr, mgr: &mut TempManager) -> InstrSet {
	todo!()
}

pub fn riscv_comp(instr: &CompInstr, mgr: &mut TempManager) -> InstrSet {
	todo!()
}

pub fn riscv_convert(instr: &ConvertInstr, mgr: &mut TempManager) -> InstrSet {
	todo!()
}

pub fn riscv_jump(instr: &JumpInstr, mgr: &mut TempManager) -> InstrSet {
	todo!()
}

pub fn riscv_cond(instr: &JumpCondInstr, mgr: &mut TempManager) -> InstrSet {
	todo!()
}

pub fn riscv_phi(instr: &PhiInstr, mgr: &mut TempManager) -> InstrSet {
	todo!()
}

pub fn riscv_ret(instr: &RetInstr, mgr: &mut TempManager) -> InstrSet {
	todo!()
}

pub fn riscv_alloc(instr: &AllocInstr, mgr: &mut TempManager) -> InstrSet {
	todo!()
}

pub fn riscv_store(instr: &StoreInstr, mgr: &mut TempManager) -> InstrSet {
	todo!()
}

pub fn riscv_load(instr: &LoadInstr, mgr: &mut TempManager) -> InstrSet {
	todo!()
}

pub fn riscv_gep(instr: &GEPInstr, mgr: &mut TempManager) -> InstrSet {
	todo!()
}

pub fn riscv_call(instr: &CallInstr, mgr: &mut TempManager) -> InstrSet {
	todo!()
}
