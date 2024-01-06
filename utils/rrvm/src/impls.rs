use std::fmt::Display;

use utils::{InstrTrait, TempTrait};

use crate::{
	cfg::CFG,
	func::RrvmFunc,
	program::{LlvmProgram, RrvmProgram},
};

impl<T: InstrTrait<U>, U: TempTrait> Display for CFG<T, U> {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(
			f,
			"{}",
			self
				.blocks
				.iter()
				.map(|v| v.borrow().to_string())
				.collect::<Vec<_>>()
				.join("\n")
		)
	}
}

impl<T: InstrTrait<U>, U: TempTrait> Display for RrvmFunc<T, U> {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let params = self
			.params
			.iter()
			.map(|v| format!("{} {}", v.get_type(), v))
			.collect::<Vec<_>>()
			.join(", ");
		let head = format!("define {} @{}({})", self.ret_type, self.name, params);
		write!(f, "{}{{\n{}\n}}", head, self.cfg)
	}
}

impl<T: InstrTrait<U>, U: TempTrait> Display for RrvmProgram<T, U> {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let funcs =
			self.funcs.iter().map(|v| v.to_string()).collect::<Vec<_>>().join("\n");

		for item in &self.global_vars {
			writeln!(f, "{:?}", &item)?;
		}

		write!(f, "{}", funcs)
	}
}

impl<T: InstrTrait<U>, U: TempTrait> RrvmProgram<T, U> {
	pub fn new() -> Self {
		Self {
			global_vars: Vec::new(),
			funcs: Vec::new(),
			next_temp: 0,
		}
	}
}

impl LlvmProgram {
	pub fn analysis(&mut self) {
		for func in self.funcs.iter() {
			func.cfg.clear_data_flow();
			func.cfg.init_phi();
			func.cfg.analysis();
		}
	}
}

impl<T: InstrTrait<U>, U: TempTrait> Default for RrvmProgram<T, U> {
	fn default() -> Self {
		Self::new()
	}
}
