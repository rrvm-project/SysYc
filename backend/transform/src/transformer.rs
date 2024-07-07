use instruction::{
	riscv::{convert::*, RiscvInstr},
	temp::TempManager,
	RiscvInstrSet,
};
use llvm::{ArithOp, LlvmInstr, LlvmInstrVariant};

use utils::errors::Result;

pub fn to_riscv(
	instr: &LlvmInstr,
	mgr: &mut TempManager,
) -> Result<RiscvInstrSet> {
	let riscv_instr = match instr.get_variant() {
		LlvmInstrVariant::ArithInstr(v) => riscv_arith(v, mgr),
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
	}?;
	Ok(riscv_instr)
}
pub fn to_rt_type(instr: &RiscvInstr) -> [i32; 5] {
	return instr.get_rtn_array();
}
