mod alloc_hoisting;
mod calc_coef;
mod code_hoisting;
mod dead_code;
mod fold_constants;
mod function_inline;
mod global_analysis;
mod global_value_numbering;
pub mod impls;
mod loops;
mod mem2reg;
mod metadata;
mod number;
mod tail_recursion;
mod unreachable;
mod useless_code;
mod useless_phis;
use metadata::MetaData;
use rrvm::program::LlvmProgram;
use utils::errors::Result;

pub trait RrvmOptimizer {
	fn new() -> Self;
	fn apply(
		self,
		program: &mut LlvmProgram,
		metadata: &mut MetaData,
	) -> Result<bool>;
}

#[derive(Default)]
pub struct Optimizer0 {}
#[derive(Default)]
pub struct Optimizer1 {}
#[derive(Default)]
pub struct Optimizer2 {}
