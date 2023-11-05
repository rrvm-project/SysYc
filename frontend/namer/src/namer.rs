use ast::tree::Program;
use anyhow::Result;

pub struct Namer{

}


impl Namer {
	pub fn transform(&self, program : Program) -> Result<Program>{
		Ok(program)	
	}
}