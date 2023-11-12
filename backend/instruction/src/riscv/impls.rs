use std::fmt::Display;

use super::riscvinstr::RiscvTriInstr;

impl Display for RiscvTriInstr {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(
			f,
			"    {} {}, {}, {}",
			self.op, self.target, self.lhs, self.rhs
		)
	}
}
