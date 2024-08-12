use rrvm::program::RiscvProgram;

mod branch_combine;
mod la_reduce;
mod modify_load_imm;

pub fn prereg_backend_optimize(program: &mut RiscvProgram, _level: i32) {
	la_reduce::la_reduce(program);
	branch_combine::branch_combine(program);
	modify_load_imm::modify_load_imm(program);
}
