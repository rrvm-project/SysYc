use ast::tree::Program;
use utils::SysycError;

pub struct  LLVMIrGen{

}


impl LLVMIrGen {
	pub fn transform(&self, program : Program) -> Result<(), SysycError>{
		Ok(())
	}
}