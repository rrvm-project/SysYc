use optimizer::remove_useless_instr;
use rrvm::program::RiscvProgram;
use stateless_cache::add_cache;

mod optimizer;
mod stateless_cache;

pub fn post_backend_optimize(program: &mut RiscvProgram, _level: i32) {
	remove_useless_instr(program);

	//should be the last!
	add_cache(program);
}
