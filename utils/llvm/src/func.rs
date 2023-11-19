use utils::Label;

use crate::{cfg::CFG, llvminstr::LlvmInstr, llvmvar::VarType, temp::Temp};
use std::fmt::Display;

pub struct LlvmFunc {
	pub label: Label,
	pub ret_type: VarType,
	pub params: Vec<Temp>,
	pub body: Vec<Box<dyn LlvmInstr>>,
	pub cfg: CFG,
}

impl Display for LlvmFunc {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "define {} {}(", self.ret_type, self.label)?;
		for (i, param) in self.params.iter().enumerate() {
			if i != 0 {
				write!(f, ", ")?;
			}
			write!(f, "{} {}", param.var_type, param)?;
		}
		writeln!(f, ") {{")?;
		// for instr in &self.body {
		// 	writeln!(f, "{}", instr)?;
		// }
		write!(f, "{}", self.cfg)?;
		writeln!(f, "}}")
	}
}
