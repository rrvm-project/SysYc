use crate::{context::IRPassContext, irpass::IRPass, svn::Svn};
use llvm::LlvmProgram;
pub struct MiddleOptimizer {}

impl MiddleOptimizer {
	pub fn optimize(self, program: &mut LlvmProgram) -> &mut LlvmProgram {
		let mut context: IRPassContext = IRPassContext {};

		let mut svn: Svn = Svn::new();

		svn.pass(program, &mut context);

		program
	}
}
