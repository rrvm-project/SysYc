use llvm::llvminstr::LlvmInstr;

pub enum InstrSet {
	LlvmInstrSet(Vec<Box<dyn LlvmInstr>>),
	RiscvInstrSet(),
}
