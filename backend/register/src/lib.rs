use instruction::temp::VarType;
use rrvm::program::RiscvProgram;
use solver::RegisterSolver;

pub mod allocator;
pub mod graph;
pub mod solver;
pub mod spill;
pub mod utils;

pub fn solve_register(program: &mut RiscvProgram) {
	for func in program.funcs.iter_mut() {
		let mut solver = RegisterSolver::new(&mut program.temp_mgr);
		solver.solve_parameter(func);
		solver.register_alloc(func, VarType::Int);
		solver.register_alloc(func, VarType::Float);
		solver.solve_caller_save(func);
		solver.memory_alloc(func);
		solver.solve_callee_save(func);
	}
}
