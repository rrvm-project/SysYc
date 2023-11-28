pub mod impls;
pub mod instr_dag;
pub mod instr_schedule;
pub mod riscv;
pub mod temp;
pub mod transformer;

use llvm::llvminstr::LlvmInstr;
use riscv::riscvinstr::RiscvInstr;

pub type LlvmInstrSet = Vec<LlvmInstr>;
pub type RiscvInstrSet = Vec<RiscvInstr>;

pub enum InstrSet {
	LlvmInstrSet(LlvmInstrSet),
	RiscvInstrSet(RiscvInstrSet),
}
