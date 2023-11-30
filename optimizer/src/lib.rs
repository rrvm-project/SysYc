mod dead_code;
pub mod impls;

use rrvm::program::LlvmProgram;
use utils::errors::Result;

pub trait RrvmOptimizer {
	fn new() -> Self;
	fn apply(self, program: &mut LlvmProgram) -> Result<()>;
}

pub struct BasicOptimizer {}
