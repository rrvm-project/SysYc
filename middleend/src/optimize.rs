use llvm::LlvmProgram;

use crate::deadcode;

pub fn optimize(mut prog: LlvmProgram) -> LlvmProgram {
	for func in prog.funcs.iter_mut() {
		deadcode::remove_dead_code(&mut func.cfg);
	}
	prog
}
