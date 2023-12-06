use allocator::RegAllocator;
use rrvm::program::RiscvFunc;

pub mod allocator;
pub mod graph;
pub mod spill;

pub fn register_alloc(func: &mut RiscvFunc) {
	RegAllocator::new().alloc(func);
}
