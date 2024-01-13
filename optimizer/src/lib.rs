mod dead_code;
mod fuyuki_vn;
pub mod impls;
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
