pub mod instr_dag;
pub mod instr_schedule;
pub mod riscv;
pub mod transformer;

use llvm::llvminstr::LlvmInstr;
use riscv::riscvinstr::RiscvInstr;

pub type LlvmInstrSet = Vec<Box<dyn LlvmInstr>>;
pub type RiscvInstrSet = Vec<Box<dyn RiscvInstr>>;

pub enum InstrSet {
	LlvmInstrSet(LlvmInstrSet),
	RiscvInstrSet(RiscvInstrSet),
}
