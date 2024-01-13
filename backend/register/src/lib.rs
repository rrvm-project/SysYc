use allocator::RegAllocator;
use rrvm::program::RiscvProgram;

pub mod allocator;
pub mod graph;
pub mod spill;
pub mod utils;

pub fn register_alloc(program: &mut RiscvProgram) {
	for func in program.funcs.iter_mut() {
		RegAllocator::default().alloc(func, &mut program.temp_mgr);
	}
}
