mod dead_code;
mod fold_constants;
mod function_inline;
mod global_value_numbering;
pub mod impls;
mod strength_reduce;
mod tail_recursion;
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
pub struct Optimizer1 {}
#[derive(Default)]
pub struct Optimizer2 {}
