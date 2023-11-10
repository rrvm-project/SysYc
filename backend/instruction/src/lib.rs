use llvm::llvminstr::LlvmInstr;

pub enum Instr {
	LlvmInstr(Box<dyn LlvmInstr>),
	RiscvInstr(),
}
