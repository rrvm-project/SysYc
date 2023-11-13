use llvm::LlvmProgram;
use rrvm_func::rrvmfunc::RrvmFunc;
use utils::SysycError;

pub struct RrvmProgram {
	pub funcs: Vec<RrvmFunc>,
}

#[allow(unused)]
impl RrvmProgram {
	pub fn new(program: LlvmProgram) -> RrvmProgram {
		todo!()
	}
	pub fn solve_global(&mut self) -> Result<(), SysycError> {
		todo!()
	}
	pub fn alloc_reg(&mut self) -> i32 {
		todo!()
	}
}
