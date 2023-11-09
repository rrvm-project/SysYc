use std::fmt::Display;

use crate::{llvminstr::*, llvmop::LlvmOp, temp::Temp, utils::all_equal};

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
		all_equal(&[
			&self.var_type,
			&self.op.oprand_type(),
			&self.lhs.get_type(),
			&self.rhs.get_type(),
		])
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

impl Display for CompInstr {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(
			f,
			"  {} = {} {} {} {}, {}",
			self.target, self.kind, self.op, self.var_type, self.lhs, self.rhs
		)
	}
}

impl LlvmInstr for CompInstr {
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
		all_equal(&[
			&self.var_type,
			&self.op.oprand_type(),
			&self.lhs.get_type(),
			&self.rhs.get_type(),
		])
	}
}

impl Display for ConvertInstr {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(
			f,
			"  {} = {} {} {} to {}",
			self.target, self.op, self.var_type, self.lhs, self.rhs
		)
	}
}

impl LlvmInstr for ConvertInstr {
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
		self.op.type_to() == self.target.var_type
			&& all_equal(&[
				&self.var_type,
				&self.op.type_from(),
				&self.lhs.get_type(),
				&self.rhs.get_type(),
			])
	}
}
