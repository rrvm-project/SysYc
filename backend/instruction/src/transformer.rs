use llvm::{temp::TempManager, LlvmInstrVariant};

use crate::{riscv::convert::*, InstrSet};

pub fn to_riscv(src: &mut InstrSet, mgr: &mut TempManager) {
	let instr = match src {
		InstrSet::LlvmInstrSet(v) => v,
		_ => unreachable!(),
	}
	.first()
	.unwrap();
	*src = match instr.get_variant() {
		LlvmInstrVariant::ArithInstr(v) => riscv_arith(v, mgr),
		LlvmInstrVariant::LabelInstr(v) => riscv_label(v, mgr),
		LlvmInstrVariant::CompInstr(v) => riscv_comp(v, mgr),
		LlvmInstrVariant::ConvertInstr(v) => riscv_convert(v, mgr),
		LlvmInstrVariant::JumpInstr(v) => riscv_jump(v, mgr),
		LlvmInstrVariant::JumpCondInstr(v) => riscv_cond(v, mgr),
		LlvmInstrVariant::PhiInstr(v) => riscv_phi(v, mgr),
		LlvmInstrVariant::RetInstr(v) => riscv_ret(v, mgr),
		LlvmInstrVariant::AllocInstr(v) => riscv_alloc(v, mgr),
		LlvmInstrVariant::StoreInstr(v) => riscv_store(v, mgr),
		LlvmInstrVariant::LoadInstr(v) => riscv_load(v, mgr),
		LlvmInstrVariant::GEPInstr(v) => riscv_gep(v, mgr),
		LlvmInstrVariant::CallInstr(v) => riscv_call(v, mgr),
	}
}
