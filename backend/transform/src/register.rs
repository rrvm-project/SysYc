use rrvm::program::RiscvFunc;

pub struct RegisterAllocer {}

impl Default for RegisterAllocer {
	fn default() -> Self {
		Self::new()
	}
}

impl RegisterAllocer {
	pub fn new() -> Self {
		Self {}
	}
	pub fn alloc(&mut self, _func: &mut RiscvFunc) {
		todo!()
	}
}
