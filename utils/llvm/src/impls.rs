use std::fmt::Display;

use utils::{Label, UseTemp};

use std::collections::HashMap;

use crate::{
	llvminstr::*, llvmop::*, utils_llvm::*, LlvmInstrVariant, LlvmTemp, VarType,
};

impl From<LlvmTemp> for Value {
	fn from(value: LlvmTemp) -> Self {
		Value::Temp(value)
	}
}

impl From<i32> for Value {
	fn from(value: i32) -> Self {
		Value::Int(value)
	}
}

impl From<usize> for Value {
	fn from(value: usize) -> Self {
		Value::Int(value as i32)
	}
}

impl From<f32> for Value {
	fn from(value: f32) -> Self {
		Value::Float(value)
	}
}

impl From<&Value> for i32 {
	fn from(value: &Value) -> Self {
		match value {
			Value::Int(v) => *v,
			_ => unreachable!(),
		}
	}
}

impl From<&Value> for LlvmTemp {
	fn from(value: &Value) -> Self {
		match value {
			Value::Temp(v) => v.clone(),
			_ => unreachable!(),
		}
	}
}

impl Display for ArithInstr {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(
			f,
			"  {} = {} {} {}, {}",
			self.target, self.op, self.var_type, self.lhs, self.rhs
		)
	}
}

impl UseTemp<LlvmTemp> for ArithInstr {
	fn get_read(&self) -> Vec<LlvmTemp> {
		unwrap_values(vec![&self.lhs, &self.rhs])
	}
	fn get_write(&self) -> Option<LlvmTemp> {
		Some(self.target.clone())
	}
}

fn map_llvm_temp(temp: &mut Value, map: &HashMap<LlvmTemp, Value>) {
	if let Value::Temp(t) = temp {
		if let Some(new_value) = map.get(t) {
			*temp = new_value.clone();
		}
	}
}

impl LlvmInstrTrait for ArithInstr {
	fn get_variant(&self) -> LlvmInstrVariant {
		LlvmInstrVariant::ArithInstr(self)
	}
	fn type_valid(&self) -> bool {
		all_equal(&[
			&self.var_type,
			&self.op.oprand_type(),
			&self.lhs.get_type(),
			&self.rhs.get_type(),
		])
	}

	fn map_temp(&mut self, map: &HashMap<LlvmTemp, Value>) {
		map_llvm_temp(&mut self.lhs, map);
		map_llvm_temp(&mut self.rhs, map);
	}

	fn replaceable(&self, map: &HashMap<LlvmTemp, Value>) -> bool {
		map.get(&self.target).is_some()
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

impl UseTemp<LlvmTemp> for CompInstr {
	fn get_read(&self) -> Vec<LlvmTemp> {
		unwrap_values(vec![&self.lhs, &self.rhs])
	}
	fn get_write(&self) -> Option<LlvmTemp> {
		Some(self.target.clone())
	}
}

impl LlvmInstrTrait for CompInstr {
	fn get_variant(&self) -> LlvmInstrVariant {
		LlvmInstrVariant::CompInstr(self)
	}
	fn type_valid(&self) -> bool {
		all_equal(&[
			&self.var_type,
			&self.op.oprand_type(),
			&self.lhs.get_type(),
			&self.rhs.get_type(),
		])
	}

	fn map_temp(&mut self, map: &HashMap<LlvmTemp, Value>) {
		map_llvm_temp(&mut self.lhs, map);
		map_llvm_temp(&mut self.rhs, map);
	}

	fn replaceable(&self, map: &HashMap<LlvmTemp, Value>) -> bool {
		map.get(&self.target).is_some()
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

impl UseTemp<LlvmTemp> for ConvertInstr {
	fn get_read(&self) -> Vec<LlvmTemp> {
		unwrap_values(vec![&self.lhs])
	}
	fn get_write(&self) -> Option<LlvmTemp> {
		Some(self.target.clone())
	}
}

impl LlvmInstrTrait for ConvertInstr {
	fn get_variant(&self) -> LlvmInstrVariant {
		LlvmInstrVariant::ConvertInstr(self)
	}
	fn type_valid(&self) -> bool {
		all_equal(&[
			&self.from_type,
			&self.op.type_from(),
			&self.lhs.get_type(),
			&self.to_type,
		])
	}

	fn map_temp(&mut self, map: &HashMap<LlvmTemp, Value>) {
		map_llvm_temp(&mut self.lhs, map);
	}

	fn replaceable(&self, map: &HashMap<LlvmTemp, Value>) -> bool {
		map.get(&self.target).is_some()
	}
}

impl Display for JumpInstr {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "  br label %{}", self.target)
	}
}

impl UseTemp<LlvmTemp> for JumpInstr {}

impl LlvmInstrTrait for JumpInstr {
	fn get_variant(&self) -> LlvmInstrVariant {
		LlvmInstrVariant::JumpInstr(self)
	}
	fn is_direct_jump(&self) -> bool {
		true
	}
	fn get_label(&self) -> Label {
		self.target.clone()
	}

	fn map_temp(&mut self, _map: &HashMap<LlvmTemp, Value>) {
		//noting to do
	}

	fn replaceable(&self, _map: &HashMap<LlvmTemp, Value>) -> bool {
		false
	}
}

impl Display for JumpCondInstr {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(
			f,
			"  br {} {}, label %{}, label %{}",
			self.var_type, self.cond, self.target_true, self.target_false
		)
	}
}

impl UseTemp<LlvmTemp> for JumpCondInstr {
	fn get_read(&self) -> Vec<LlvmTemp> {
		unwrap_values(vec![&self.cond])
	}
}

impl LlvmInstrTrait for JumpCondInstr {
	fn get_variant(&self) -> LlvmInstrVariant {
		LlvmInstrVariant::JumpCondInstr(self)
	}
	fn type_valid(&self) -> bool {
		all_equal(&[&self.cond.get_type(), &self.var_type, &VarType::I32])
	}
	fn new_jump(&self) -> Option<JumpInstr> {
		if self.cond.always_true() {
			return Some(JumpInstr {
				target: self.target_true.clone(),
			});
		}
		if self.cond.always_false() {
			return Some(JumpInstr {
				target: self.target_false.clone(),
			});
		}
		if self.target_true == self.target_false {
			return Some(JumpInstr {
				target: self.target_false.clone(),
			});
		}
		None
	}
	fn is_jump_cond(&self) -> bool {
		true
	}

	fn map_temp(&mut self, map: &HashMap<LlvmTemp, Value>) {
		map_llvm_temp(&mut self.cond, map);
	}

	fn replaceable(&self, _map: &HashMap<LlvmTemp, Value>) -> bool {
		false
	}
}

impl Display for PhiInstr {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let ctx: Vec<_> = self
			.source
			.iter()
			.map(|(a, b)| format!("[{}, label %{}]", a, b))
			.collect();
		write!(
			f,
			"  {} = phi {} {}",
			self.target,
			self.var_type,
			ctx.join(", ")
		)
	}
}

impl UseTemp<LlvmTemp> for PhiInstr {
	fn get_read(&self) -> Vec<LlvmTemp> {
		self.source.iter().flat_map(|(v, _)| v.unwrap_temp()).collect()
	}
	fn get_write(&self) -> Option<LlvmTemp> {
		Some(self.target.clone())
	}
}

impl LlvmInstrTrait for PhiInstr {
	fn get_variant(&self) -> LlvmInstrVariant {
		LlvmInstrVariant::PhiInstr(self)
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
	fn map_temp(&mut self, map: &HashMap<LlvmTemp, Value>) {
		for (value, _label) in &mut self.source {
			map_llvm_temp(value, map);
		}
	}
	fn replaceable(&self, _map: &HashMap<LlvmTemp, Value>) -> bool {
		false
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

impl UseTemp<LlvmTemp> for RetInstr {
	fn get_read(&self) -> Vec<LlvmTemp> {
		self.value.as_ref().map_or(Vec::new(), |v| {
			vec![&v].into_iter().flat_map(|v| v.unwrap_temp()).collect()
		})
	}
}

impl LlvmInstrTrait for RetInstr {
	fn get_variant(&self) -> LlvmInstrVariant {
		LlvmInstrVariant::RetInstr(self)
	}
	fn is_ret(&self) -> bool {
		true
	}
	fn map_temp(&mut self, map: &HashMap<LlvmTemp, Value>) {
		if let Some(value) = &mut self.value {
			map_llvm_temp(value, map);
		}
	}
	fn replaceable(&self, _map: &HashMap<LlvmTemp, Value>) -> bool {
		false
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

impl UseTemp<LlvmTemp> for AllocInstr {
	fn get_read(&self) -> Vec<LlvmTemp> {
		vec![&self.length].into_iter().flat_map(|v| v.unwrap_temp()).collect()
	}
	fn get_write(&self) -> Option<LlvmTemp> {
		Some(self.target.clone())
	}
}

impl LlvmInstrTrait for AllocInstr {
	fn get_variant(&self) -> LlvmInstrVariant {
		LlvmInstrVariant::AllocInstr(self)
	}
	fn type_valid(&self) -> bool {
		self.length.get_type() == VarType::I32
	}
	fn get_alloc(&self) -> Option<(LlvmTemp, Value)> {
		Some((self.target.clone(), self.length.clone()))
	}
	fn map_temp(&mut self, _map: &HashMap<LlvmTemp, Value>) {}
	fn replaceable(&self, _map: &HashMap<LlvmTemp, Value>) -> bool {
		false
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

impl UseTemp<LlvmTemp> for StoreInstr {
	fn get_read(&self) -> Vec<LlvmTemp> {
		unwrap_values(vec![&self.value, &self.addr])
	}
}

impl LlvmInstrTrait for StoreInstr {
	fn get_variant(&self) -> LlvmInstrVariant {
		LlvmInstrVariant::StoreInstr(self)
	}
	fn type_valid(&self) -> bool {
		type_match_ptr(self.value.get_type(), self.addr.get_type())
	}
	fn has_sideeffect(&self) -> bool {
		true
	}
	fn is_store(&self) -> bool {
		true
	}
	fn map_temp(&mut self, map: &HashMap<LlvmTemp, Value>) {
		map_llvm_temp(&mut self.value, map);
		map_llvm_temp(&mut self.addr, map);
	}
	fn replaceable(&self, _map: &HashMap<LlvmTemp, Value>) -> bool {
		false
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

impl UseTemp<LlvmTemp> for LoadInstr {
	fn get_read(&self) -> Vec<LlvmTemp> {
		vec![&self.addr].into_iter().flat_map(|v| v.unwrap_temp()).collect()
	}
	fn get_write(&self) -> Option<LlvmTemp> {
		Some(self.target.clone())
	}
}

impl LlvmInstrTrait for LoadInstr {
	fn get_variant(&self) -> LlvmInstrVariant {
		LlvmInstrVariant::LoadInstr(self)
	}
	fn type_valid(&self) -> bool {
		type_match_ptr(self.var_type, self.addr.get_type())
	}
	fn is_load(&self) -> bool {
		self.addr.unwrap_temp().map_or(true, |v| !v.is_global)
	}
	fn map_temp(&mut self, map: &HashMap<LlvmTemp, Value>) {
		map_llvm_temp(&mut self.addr, map);
	}
	fn replaceable(&self, _map: &HashMap<LlvmTemp, Value>) -> bool {
		false
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

impl UseTemp<LlvmTemp> for GEPInstr {
	fn get_read(&self) -> Vec<LlvmTemp> {
		unwrap_values(vec![&self.addr, &self.offset])
	}
	fn get_write(&self) -> Option<LlvmTemp> {
		Some(self.target.clone())
	}
}

impl LlvmInstrTrait for GEPInstr {
	fn get_variant(&self) -> LlvmInstrVariant {
		LlvmInstrVariant::GEPInstr(self)
	}
	fn type_valid(&self) -> bool {
		is_ptr(self.addr.get_type())
			&& self.offset.get_type() == VarType::I32
			&& type_match_ptr(self.var_type, self.addr.get_type())
	}
	fn map_temp(&mut self, map: &HashMap<LlvmTemp, Value>) {
		map_llvm_temp(&mut self.addr, map);
		map_llvm_temp(&mut self.offset, map);
	}
	fn replaceable(&self, _map: &HashMap<LlvmTemp, Value>) -> bool {
		false
	}
}

impl Display for CallInstr {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let ctx: Vec<_> =
			self.params.iter().map(|(a, b)| format!("{} {}", a, b)).collect();
		write!(
			f,
			"  {} = call {} @{}({})",
			self.target,
			self.var_type,
			self.func,
			ctx.join(", ")
		)
	}
}

impl UseTemp<LlvmTemp> for CallInstr {
	fn get_read(&self) -> Vec<LlvmTemp> {
		unwrap_values(self.params.iter().map(|(_, x)| x).collect())
	}
	fn get_write(&self) -> Option<LlvmTemp> {
		Some(self.target.clone())
	}
}

impl LlvmInstrTrait for CallInstr {
	fn get_variant(&self) -> LlvmInstrVariant {
		LlvmInstrVariant::CallInstr(self)
	}
	fn has_sideeffect(&self) -> bool {
		true
	}
	fn is_call(&self) -> bool {
		true
	}
	fn map_temp(&mut self, map: &HashMap<LlvmTemp, Value>) {
		for (_, value) in &mut self.params {
			map_llvm_temp(value, map);
		}
	}
	fn replaceable(&self, _map: &HashMap<LlvmTemp, Value>) -> bool {
		false
	}
}
