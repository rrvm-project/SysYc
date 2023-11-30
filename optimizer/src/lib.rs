mod dead_code;
pub mod impls;
mod unreachable;

use rrvm::program::LlvmProgram;
use utils::errors::Result;

use dead_code::RemoveDeadCode;
use unreachable::RemoveUnreachCode;

pub trait RrvmOptimizer {
	fn new() -> Self;
	fn apply(self, program: &mut LlvmProgram) -> Result<()>;
}

pub struct Optimizer0 {}
pub struct Optimizer1 {}
pub struct Optimizer2 {}
