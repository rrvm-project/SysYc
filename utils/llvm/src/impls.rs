use std::fmt::Display;

use crate::{llvminstr::*, temp::Temp};

impl Display for ArithInstr {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(
			f,
			"  {} = {} {} {}, {}",
			self.target, self.op, self.var_type, self.lhs, self.rhs
		)
	}
}

impl LlvmInstr for ArithInstr {
	fn get_read(&self) -> Vec<Temp> {
		vec![&self.lhs, &self.rhs]
			.into_iter()
			.flat_map(|v| v.unwrap_temp())
			.collect()
	}
	fn get_write(&self) -> Vec<Temp> {
		vec![self.target.clone()]
	}
	fn is_label(&self) -> bool {
		false
	}
	fn is_seq(&self) -> bool {
		true
	}
	fn type_valid(&self) -> bool {
		self.var_type == self.op.oprand_type()
			&& self.lhs.unwrap_temp().map_or(true, |v| self.var_type == v.var_type)
			&& self.rhs.unwrap_temp().map_or(true, |v| self.var_type == v.var_type)
	}
}

impl Display for LabelInstr {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{}:", self.label.name)
	}
}

impl LlvmInstr for LabelInstr {
	fn get_read(&self) -> Vec<Temp> {
		Vec::new()
	}
	fn get_write(&self) -> Vec<Temp> {
		Vec::new()
	}
	fn is_label(&self) -> bool {
		true
	}
	fn is_seq(&self) -> bool {
		false
	}
	fn type_valid(&self) -> bool {
		true
	}
}
