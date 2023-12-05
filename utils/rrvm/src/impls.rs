use std::fmt::Display;

use utils::UseTemp;

use crate::{cfg::CFG, func::RrvmFunc, program::RrvmProgram};

impl<T: Display + UseTemp<U>, U: Display> Display for CFG<T, U> {
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

impl<T: Display + UseTemp<U>, U: Display> Display for RrvmFunc<T, U> {
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

impl<T: Display + UseTemp<U>, U: Display> Display for RrvmProgram<T, U> {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let funcs =
			self.funcs.iter().map(|v| v.to_string()).collect::<Vec<_>>().join("\n");

		write!(f, "{}", funcs)
	}
}
