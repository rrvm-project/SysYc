use rrvm::program::LlvmProgram;
pub mod impls;

pub trait RrvmOptimizer {
	fn new() -> Self;
	fn apply(self, program: LlvmProgram) -> LlvmProgram;
}

pub struct BasicOptimizer {}
