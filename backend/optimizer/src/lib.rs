use optimizer::remove_useless_instr;
use rrvm::program::RiscvProgram;

mod optimizer;

pub fn backend_optimize(program: &mut RiscvProgram, _level: i32) {
	remove_useless_instr(program);
}
