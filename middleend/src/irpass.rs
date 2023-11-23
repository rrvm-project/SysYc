use crate::context::IRPassContext;
use llvm::LlvmProgram;

pub trait IRPass {
	fn pass(&mut self, program: &mut LlvmProgram, context: &mut IRPassContext);
}
