mod dead_code;
mod fold_constants;
mod function_inline;
mod fuyuki_vn;
pub mod impls;
mod tail_recursion;
mod strength_reduce;
mod unreachable;
mod useless_code;
mod useless_phis;
use rrvm::program::LlvmProgram;
use utils::errors::Result;

pub trait RrvmOptimizer {
	fn new() -> Self;
	fn apply(self, program: &mut LlvmProgram) -> Result<bool>;
}

#[derive(Default)]
pub struct Optimizer0 {}
#[derive(Default)]
pub struct Optimizer1 {
	_strength_reduce_total_new_temp: u32,
}
#[derive(Default)]
pub struct Optimizer2 {
	strength_reduce_total_new_temp: u32,
}
