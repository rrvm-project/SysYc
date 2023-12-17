mod dead_code;
pub mod impls;
mod unreachable;

use rrvm::program::LlvmProgram;
use utils::errors::Result;

use dead_code::RemoveDeadCode;
use unreachable::RemoveUnreachCode;

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
