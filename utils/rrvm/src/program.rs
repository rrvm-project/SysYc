use crate::func::RrvmFunc;

pub struct RrvmProgram {
	// pub global_vars: Vec<>
	pub funcs: Vec<RrvmFunc>,
}

impl RrvmProgram {
	pub fn new() -> Self {
		Self { funcs: Vec::new() }
	}
}

impl Default for RrvmProgram {
	fn default() -> Self {
		Self::new()
	}
}
