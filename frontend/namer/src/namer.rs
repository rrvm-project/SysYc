use ast::tree::Program;
use utils::SysycError;

#[derive(Debug)]
pub struct Namer{
	pub loop_num: i32,
}

impl Default for Namer {
	fn default() -> Self {
		Namer {
			loop_num: 0,
		}
	}	
}

impl Namer {
	pub fn transform(&self, program : Program) -> Result<Program, SysycError>{
		Ok(program)	
	}
}