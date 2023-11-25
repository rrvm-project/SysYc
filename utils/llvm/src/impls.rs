use std::fmt::Display;

use utils::Label;

use crate::{
	llvminstr::*,
	llvmop::*,
	llvmvar::VarType,
	temp::Temp,
	utils_llvm::{all_equal, is_ptr, type_match_ptr, unwrap_values},
	LlvmInstrVariant,
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
	fn get_variant(&self) -> LlvmInstrVariant {
		LlvmInstrVariant::ArithInstr(self)
	}
	fn get_read(&self) -> Vec<Temp> {
		unwrap_values(vec![&self.lhs, &self.rhs])
	}
	fn get_write(&self) -> Option<Temp> {
		Some(self.target.clone())
	}
	fn type_valid(&self) -> bool {
		all_equal(&[
			&self.var_type,
			&self.op.oprand_type(),
			&self.lhs.get_type(),
			&self.rhs.get_type(),
		])
	}
	fn swap_temp(&mut self, old: Temp, new: Value) {
		if self.lhs.unwrap_temp().map_or(false, |t| t == old) {
			self.lhs = new.clone();
		}
		if self.rhs.unwrap_temp().map_or(false, |t| t == old) {
			self.rhs = new;
		}
	}

	fn replace_temp(&mut self, map: &std::collections::HashMap<Temp, Value>) {
		if let Value::Temp(temp) = &self.lhs {
			if let Some(value) = map.get(temp) {
				self.lhs = value.clone();
			}
		}

		if let Value::Temp(temp) = &self.rhs {
			if let Some(value) = map.get(temp) {
				self.rhs = value.clone();
			}
		}
	}
}

impl Display for LabelInstr {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{}:", self.label.name)
	}
}

impl LlvmInstr for LabelInstr {
	fn get_variant(&self) -> LlvmInstrVariant {
		LlvmInstrVariant::LabelInstr(self)
	}
	fn get_label(&self) -> Option<Label> {
		Some(self.label.clone())
	}
	fn swap_temp(&mut self, _old: Temp, _new: Value) {
		// do nothing
	}

	fn replace_temp(&mut self, _map: &std::collections::HashMap<Temp, Value>) {}
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
	fn get_variant(&self) -> LlvmInstrVariant {
		LlvmInstrVariant::CompInstr(self)
	}
	fn get_read(&self) -> Vec<Temp> {
		unwrap_values(vec![&self.lhs, &self.rhs])
	}
	fn get_write(&self) -> Option<Temp> {
		Some(self.target.clone())
	}
	fn type_valid(&self) -> bool {
		all_equal(&[
			&self.var_type,
			&self.op.oprand_type(),
			&self.lhs.get_type(),
			&self.rhs.get_type(),
		])
	}

	fn swap_temp(&mut self, old: Temp, new: Value) {
		if self.lhs.unwrap_temp().map_or(false, |t| t == old) {
			self.lhs = new.clone();
		}
		if self.rhs.unwrap_temp().map_or(false, |t| t == old) {
			self.rhs = new;
		}
	}

	fn replace_temp(&mut self, map: &std::collections::HashMap<Temp, Value>) {
		if let Value::Temp(temp) = &self.lhs {
			if let Some(value) = map.get(temp) {
				self.lhs = value.clone();
			}
		}
		if let Value::Temp(temp) = &self.rhs {
			if let Some(value) = map.get(temp) {
				self.rhs = value.clone();
			}
		}
	}
}

impl Display for ConvertInstr {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(
			f,
			"  {} = {} {} {} to {}",
			self.target, self.op, self.from_type, self.lhs, self.to_type
		)
	}
}

impl LlvmInstr for ConvertInstr {
	fn get_variant(&self) -> LlvmInstrVariant {
		LlvmInstrVariant::ConvertInstr(self)
	}
	fn get_read(&self) -> Vec<Temp> {
		unwrap_values(vec![&self.lhs])
	}
	fn get_write(&self) -> Option<Temp> {
		Some(self.target.clone())
	}
	fn type_valid(&self) -> bool {
		all_equal(&[
			&self.from_type,
			&self.op.type_from(),
			&self.lhs.get_type(),
			&self.to_type,
		])
	}
	fn swap_temp(&mut self, old: Temp, new: Value) {
		// omit target
		if self.lhs.unwrap_temp().map_or(false, |t| t == old) {
			self.lhs = new;
		}
	}

	fn replace_temp(&mut self, map: &std::collections::HashMap<Temp, Value>) {
		if let Value::Temp(temp) = &self.lhs {
			if let Some(value) = map.get(temp) {
				self.lhs = value.clone();
			}
		}
	}
}

impl Display for JumpInstr {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "  br label {}", self.target)
	}
}

impl LlvmInstr for JumpInstr {
	fn get_variant(&self) -> LlvmInstrVariant {
		LlvmInstrVariant::JumpInstr(self)
	}
	fn is_seq(&self) -> bool {
		false
	}
	fn swap_temp(&mut self, _old: Temp, _new: Value) {
		// do nothing
	}

	fn replace_temp(&mut self, _map: &std::collections::HashMap<Temp, Value>) {}
}

impl Display for JumpCondInstr {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(
			f,
			"  br {} {}, label {}, label {}",
			self.var_type, self.cond, self.target_true, self.target_false
		)
	}
}

impl LlvmInstr for JumpCondInstr {
	fn get_variant(&self) -> LlvmInstrVariant {
		LlvmInstrVariant::JumpCondInstr(self)
	}
	fn is_seq(&self) -> bool {
		false
	}
	fn type_valid(&self) -> bool {
		all_equal(&[&self.cond.get_type(), &self.var_type, &VarType::I32])
	}
	fn swap_temp(&mut self, old: Temp, new: Value) {
		if self.cond.unwrap_temp().map_or(false, |t| t == old) {
			self.cond = new;
		}
	}
	fn replace_temp(&mut self, map: &std::collections::HashMap<Temp, Value>) {
		if let Value::Temp(temp) = &self.cond {
			if let Some(value) = map.get(temp) {
				self.cond = value.clone();
			}
		}
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
	fn get_variant(&self) -> LlvmInstrVariant {
		LlvmInstrVariant::PhiInstr(self)
	}
	fn get_read(&self) -> Vec<Temp> {
		self.source.iter().flat_map(|(v, _)| v.unwrap_temp()).collect()
	}
	fn get_write(&self) -> Option<Temp> {
		Some(self.target.clone())
	}
	fn type_valid(&self) -> bool {
		let mut v: Vec<_> = self.source.iter().map(|(v, _)| v.get_type()).collect();
		v.push(self.var_type);
		v.push(self.target.var_type);
		all_equal(&v)
	}
	fn is_phi(&self) -> bool {
		true
	}
	fn swap_temp(&mut self, old: Temp, new: Value) {
		// do nothing
		let mut new_source = vec![];
		for (value, label) in std::mem::take(&mut self.source) {
			if let Value::Temp(temp) = &value {
				if *temp == old {
					new_source.push((new.clone(), label));
					continue;
				}
			}
			new_source.push((value, label));
		}
		self.source = new_source;
	}

	fn replace_temp(&mut self, map: &std::collections::HashMap<Temp, Value>) {
		let mut new_source = vec![];
		for (value, label) in std::mem::take(&mut self.source) {
			if let Value::Temp(temp) = &value {
				if let Some(new_value) = map.get(temp) {
					new_source.push((new_value.clone(), label));
					continue;
				}
			}
			new_source.push((value, label));
		}
		self.source = new_source;
	}
}

impl Display for RetInstr {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		if let Some(value) = self.value.as_ref() {
			write!(f, "  ret {} {}", value.get_type(), value)
		} else {
			write!(f, "  ret void")
		}
	}
}

impl LlvmInstr for RetInstr {
	fn get_variant(&self) -> LlvmInstrVariant {
		LlvmInstrVariant::RetInstr(self)
	}
	fn get_read(&self) -> Vec<Temp> {
		self.value.as_ref().map_or(Vec::new(), |v| {
			vec![&v].into_iter().flat_map(|v| v.unwrap_temp()).collect()
		})
	}
	fn is_seq(&self) -> bool {
		false
	}
	fn is_ret(&self) -> bool {
		true
	}
	fn swap_temp(&mut self, old: Temp, new: Value) {
		if self
			.value
			.clone()
			.map_or(false, |v| v.unwrap_temp().map_or(false, |t| t == old))
		{
			self.value = Some(new);
		}
	}

	fn replace_temp(&mut self, map: &std::collections::HashMap<Temp, Value>) {
		if let Some(Value::Temp(temp)) = &self.value {
			if let Some(value) = map.get(temp) {
				self.value = Some(value.clone());
			}
		}
	}
}

impl Display for AllocInstr {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(
			f,
			"  {} = alloca {}, {} {}",
			self.target,
			self.var_type,
			self.length.get_type(),
			self.length
		)
	}
}

impl LlvmInstr for AllocInstr {
	fn get_variant(&self) -> LlvmInstrVariant {
		LlvmInstrVariant::AllocInstr(self)
	}
	fn get_read(&self) -> Vec<Temp> {
		vec![&self.length].into_iter().flat_map(|v| v.unwrap_temp()).collect()
	}
	fn get_write(&self) -> Option<Temp> {
		Some(self.target.clone())
	}
	fn type_valid(&self) -> bool {
		self.length.get_type() == VarType::I32
	}
	fn swap_temp(&mut self, old: Temp, new: Value) {
		if self.length.unwrap_temp().map_or(false, |t| t == old) {
			self.length = new;
		}
	}
	fn replace_temp(&mut self, map: &std::collections::HashMap<Temp, Value>) {
		if let Value::Temp(temp) = &self.length {
			if let Some(value) = map.get(temp) {
				self.length = value.clone();
			}
		}
	}
}

impl Display for StoreInstr {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(
			f,
			"  store {} {}, {} {}",
			self.value.get_type(),
			self.value,
			self.addr.get_type(),
			self.addr
		)
	}
}

impl LlvmInstr for StoreInstr {
	fn get_variant(&self) -> LlvmInstrVariant {
		LlvmInstrVariant::StoreInstr(self)
	}
	fn get_read(&self) -> Vec<Temp> {
		unwrap_values(vec![&self.value, &self.addr])
	}
	fn type_valid(&self) -> bool {
		type_match_ptr(self.value.get_type(), self.addr.get_type())
	}
	fn swap_temp(&mut self, old: Temp, new: Value) {
		if self.value.unwrap_temp().map_or(false, |t| t == old) {
			self.value = new.clone();
		}
		if self.addr.unwrap_temp().map_or(false, |t| t == old) {
			self.addr = new;
		}
	}
	fn replace_temp(&mut self, map: &std::collections::HashMap<Temp, Value>) {
		if let Value::Temp(temp) = &self.value {
			if let Some(value) = map.get(temp) {
				self.value = value.clone();
			}
		}
		if let Value::Temp(temp) = &self.addr {
			if let Some(value) = map.get(temp) {
				self.addr = value.clone();
			}
		}
	}
}

impl Display for LoadInstr {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(
			f,
			"  {} = load {}, {} {}",
			self.target,
			self.var_type,
			self.addr.get_type(),
			self.addr
		)
	}
}

impl LlvmInstr for LoadInstr {
	fn get_variant(&self) -> LlvmInstrVariant {
		LlvmInstrVariant::LoadInstr(self)
	}
	fn get_read(&self) -> Vec<Temp> {
		vec![&self.addr].into_iter().flat_map(|v| v.unwrap_temp()).collect()
	}
	fn get_write(&self) -> Option<Temp> {
		Some(self.target.clone())
	}
	fn type_valid(&self) -> bool {
		type_match_ptr(self.var_type, self.addr.get_type())
	}
	fn swap_temp(&mut self, old: Temp, new: Value) {
		if self.addr.unwrap_temp().map_or(false, |t| t == old) {
			self.addr = new;
		}
	}
	fn replace_temp(&mut self, map: &std::collections::HashMap<Temp, Value>) {
		if let Value::Temp(temp) = &self.addr {
			if let Some(value) = map.get(temp) {
				self.addr = value.clone();
			}
		}
	}
}

impl Display for GEPInstr {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(
			f,
			"  {} = getelementptr {} {}, {} {}",
			self.target,
			self.addr.get_type(),
			self.addr,
			self.offset.get_type(),
			self.offset
		)
	}
}

impl LlvmInstr for GEPInstr {
	fn get_variant(&self) -> LlvmInstrVariant {
		LlvmInstrVariant::GEPInstr(self)
	}
	fn get_read(&self) -> Vec<Temp> {
		unwrap_values(vec![&self.addr, &self.offset])
	}
	fn get_write(&self) -> Option<Temp> {
		Some(self.target.clone())
	}
	fn type_valid(&self) -> bool {
		is_ptr(self.addr.get_type())
			&& self.offset.get_type() == VarType::I32
			&& type_match_ptr(self.var_type, self.addr.get_type())
	}
	fn swap_temp(&mut self, old: Temp, new: Value) {
		if self.addr.unwrap_temp().map_or(false, |t| t == old) {
			self.addr = new.clone();
		}
		if self.offset.unwrap_temp().map_or(false, |t| t == old) {
			self.offset = new;
		}
	}

	fn replace_temp(&mut self, map: &std::collections::HashMap<Temp, Value>) {
		if let Value::Temp(temp) = &self.addr {
			if let Some(value) = map.get(temp) {
				self.addr = value.clone();
			}
		}
		if let Value::Temp(temp) = &self.offset {
			if let Some(value) = map.get(temp) {
				self.offset = value.clone();
			}
		}
	}
}

impl Display for CallInstr {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let ctx: Vec<_> =
			self.params.iter().map(|(a, b)| format!("{} {}", a, b)).collect();
		write!(
			f,
			"  {} = call {} {}({})",
			self.target,
			self.var_type,
			self.func,
			ctx.join(", ")
		)
	}
}

impl LlvmInstr for CallInstr {
	fn get_variant(&self) -> LlvmInstrVariant {
		LlvmInstrVariant::CallInstr(self)
	}
	fn get_read(&self) -> Vec<Temp> {
		unwrap_values(self.params.iter().map(|(_, x)| x).collect())
	}
	fn get_write(&self) -> Option<Temp> {
		Some(self.target.clone())
	}
	fn swap_temp(&mut self, old: Temp, new: Value) {
		for (_, v) in &mut self.params {
			if v.unwrap_temp().map_or(false, |t| t == old) {
				*v = new.clone();
			}
		}
	}

	fn replace_temp(&mut self, map: &std::collections::HashMap<Temp, Value>) {
		for (_, v) in &mut self.params {
			if let Value::Temp(temp) = v {
				if let Some(value) = map.get(temp) {
					*v = value.clone();
				}
			}
		}
	}
}
