use ast::tree::Program;
use utils::SysycError;

pub struct LLVMIrGen {}

#[allow(unused_variables)]
impl LLVMIrGen {
	pub fn transform(&self, program: Program) -> Result<(), SysycError> {
		Ok(())
	}
}
