use std::fmt::Display;

use crate::{
	llvminstr::*, llvmop::*, llvmvar::VarType, temp::Temp, utils::all_equal,
};

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
	fn is_label(&self) -> bool {
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
	fn type_valid(&self) -> bool {
		all_equal(&[
			&self.var_type,
			&self.op.type_from(),
			&self.lhs.get_type(),
			&self.rhs.get_type(),
		])
	}
}

impl Display for JumpInstr {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "  br label {}", self.target)
	}
}

impl LlvmInstr for JumpInstr {
	fn is_seq(&self) -> bool {
		false
	}
}

impl Display for JumpCondInstr {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(
			f,
			"  br {} {}, label {}, label {}",
			self.var_type, self.cond, self.target_true, self.target_true
		)
	}
}

impl LlvmInstr for JumpCondInstr {
	fn is_seq(&self) -> bool {
		false
	}
	fn type_valid(&self) -> bool {
		all_equal(&[&self.cond.get_type(), &self.var_type, &VarType::I32])
	}
}

impl Display for PhiInstr {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let ctx: Vec<_> =
			self.source.iter().map(|(a, b)| format!("[{}, {}]", a, b)).collect();
		write!(
			f,
			"  {} = phi {} {}",
			self.target,
			self.var_type,
			ctx.join(", ")
		)
	}
}

impl LlvmInstr for PhiInstr {
	fn get_read(&self) -> Vec<Temp> {
		self.source.iter().flat_map(|(v, _)| v.unwrap_temp()).collect()
	}
	fn type_valid(&self) -> bool {
		let mut v: Vec<_> = self.source.iter().map(|(v, _)| v.get_type()).collect();
		v.push(self.var_type);
		v.push(self.target.var_type);
		all_equal(&v)
	}
}

impl Display for RetInstr {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		if let Value::Void = self.value {
			write!(f, "  ret void")
		} else {
			write!(f, "  ret {} {}", self.value.get_type(), self.value)
		}
	}
}

impl LlvmInstr for RetInstr {
	fn get_read(&self) -> Vec<Temp> {
		vec![&self.value].into_iter().flat_map(|v| v.unwrap_temp()).collect()
	}
	fn is_seq(&self) -> bool {
		false
	}
	fn is_ret(&self) -> bool {
		true
	}
}
