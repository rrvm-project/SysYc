use rrvm::program::RiscvProgram;

mod la_reduce;
pub fn prereg_backend_optimize(program: &mut RiscvProgram, _level: i32) {
	la_reduce::la_reduce(program);
}
