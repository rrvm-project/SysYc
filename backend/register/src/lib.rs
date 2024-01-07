use allocator::RegAllocator;
use rrvm::program::RiscvFunc;

pub mod allocator;
pub mod graph;
pub mod spill;
pub mod utils;

pub fn register_alloc(func: &mut RiscvFunc) {
	func.cfg.analysis();
	println!("{}", func);
	RegAllocator::default().alloc(func);
}
