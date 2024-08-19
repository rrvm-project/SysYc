use rrvm::program::RiscvProgram;

mod branch_combine;
mod instruction_scheduling;
mod la_reduce;
mod modify_load_imm;
mod shift_add;

pub fn prereg_backend_optimize(program: &mut RiscvProgram, _level: i32) {
	branch_combine::branch_combine(program);
	modify_load_imm::modify_load_imm(program);
	la_reduce::la_reduce(program);
	shift_add::shift_add(program);
	instruction_scheduling::instr_schedule_program(program);
}
