use llvm::llvminstr::LlvmInstr;

#[allow(unused)]
pub struct SelectionDag {
	instr: Vec<Box<dyn LlvmInstr>>,
}
