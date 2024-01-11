use std::fmt::Display;

use utils::{Label, UseTemp};

use std::collections::HashMap;

use crate::{
	llvminstr::*,
	llvmop::*,
	llvmvar::VarType,
	temp::Temp,
	utils_llvm::{all_equal, is_ptr, type_match_ptr, unwrap_values},
	LlvmInstrVariant,
};

impl From<Temp> for Value {
	fn from(value: Temp) -> Self {
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

impl From<&Value> for Temp {
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

impl UseTemp<Temp> for ArithInstr {
	fn get_read(&self) -> Vec<Temp> {
		unwrap_values(vec![&self.lhs, &self.rhs])
	}
	fn get_write(&self) -> Option<Temp> {
		Some(self.target.clone())
	}
}

fn map_llvm_temp(temp: &mut Value, map: &HashMap<Temp, Value>) {
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
	fn replace_read(&mut self, old: Temp, new: Value) {
		if self.lhs == Value::Temp(old.clone()) {
			self.lhs = new.clone();
		}
		if self.rhs == Value::Temp(old.clone()) {
			self.rhs = new;
		}
	}

	fn map_temp(&mut self, map: &HashMap<Temp, Value>) {
		map_llvm_temp(&mut self.lhs, map);
		map_llvm_temp(&mut self.rhs, map);
	}

	fn replaceable(&self, map: &HashMap<Temp, Value>) -> bool {
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

impl UseTemp<Temp> for CompInstr {
	fn get_read(&self) -> Vec<Temp> {
		unwrap_values(vec![&self.lhs, &self.rhs])
	}
	fn get_write(&self) -> Option<Temp> {
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
	fn replace_read(&mut self, old: Temp, new: Value) {
		if self.lhs == Value::Temp(old.clone()) {
			self.lhs = new.clone();
		}
		if self.rhs == Value::Temp(old.clone()) {
			self.rhs = new;
		}
	}

	fn map_temp(&mut self, map: &HashMap<Temp, Value>) {
		map_llvm_temp(&mut self.lhs, map);
		map_llvm_temp(&mut self.rhs, map);
	}

	fn replaceable(&self, map: &HashMap<Temp, Value>) -> bool {
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

impl UseTemp<Temp> for ConvertInstr {
	fn get_read(&self) -> Vec<Temp> {
		unwrap_values(vec![&self.lhs])
	}
	fn get_write(&self) -> Option<Temp> {
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
	fn replace_read(&mut self, old: Temp, new: Value) {
		if self.lhs == Value::Temp(old.clone()) {
			self.lhs = new;
		}
	}

	fn map_temp(&mut self, map: &HashMap<Temp, Value>) {
		map_llvm_temp(&mut self.lhs, map);
	}

	fn replaceable(&self, map: &HashMap<Temp, Value>) -> bool {
		map.get(&self.target).is_some()
	}
}

impl Display for JumpInstr {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "  br label %{}", self.target)
	}
}

impl UseTemp<Temp> for JumpInstr {}

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

	fn map_temp(&mut self, _map: &HashMap<Temp, Value>) {
		//noting to do
	}

	fn replaceable(&self, _map: &HashMap<Temp, Value>) -> bool {
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

impl UseTemp<Temp> for JumpCondInstr {
	fn get_read(&self) -> Vec<Temp> {
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
	fn replace_read(&mut self, old: Temp, new: Value) {
		if self.cond == Value::Temp(old.clone()) {
			self.cond = new;
		}
	}

	fn map_temp(&mut self, map: &HashMap<Temp, Value>) {
		map_llvm_temp(&mut self.cond, map);
	}

	fn replaceable(&self, _map: &HashMap<Temp, Value>) -> bool {
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

impl UseTemp<Temp> for PhiInstr {
	fn get_read(&self) -> Vec<Temp> {
		self.source.iter().flat_map(|(v, _)| v.unwrap_temp()).collect()
	}
	fn get_write(&self) -> Option<Temp> {
		Some(self.target.clone())
	}
}

impl PhiInstr {
	pub fn get_read_with_label(&self) -> Vec<(Temp, Label)> {
		self
			.source
			.iter()
			.flat_map(|(v, l)| v.unwrap_temp().map(|v| (v, l.clone())))
			.collect()
	}
	pub fn all_has_the_same_value(&self) -> Option<Value> {
		let v = self.source.iter().map(|(v, _)| v.clone()).collect::<Vec<_>>();
		if !all_equal::<Value>(&v) {
			return None;
		}
		Some(v[0].clone())
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
	fn replace_read(&mut self, old: Temp, new: Value) {
		for (v, _) in self.source.iter_mut() {
			if v == &Value::Temp(old.clone()) {
				*v = new.clone();
			}
		}
	}
	fn map_temp(&mut self, map: &HashMap<Temp, Value>) {
		for (value, _label) in &mut self.source {
			map_llvm_temp(value, map);
		}
	}

	fn replaceable(&self, _map: &HashMap<Temp, Value>) -> bool {
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

impl UseTemp<Temp> for RetInstr {
	fn get_read(&self) -> Vec<Temp> {
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
	fn replace_read(&mut self, old: Temp, new: Value) {
		if self.value == Some(Value::Temp(old.clone())) {
			self.value = Some(new);
		}
	}
	fn map_temp(&mut self, map: &HashMap<Temp, Value>) {
		if let Some(value) = &mut self.value {
			map_llvm_temp(value, map);
		}
	}

	fn replaceable(&self, _map: &HashMap<Temp, Value>) -> bool {
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

impl UseTemp<Temp> for AllocInstr {
	fn get_read(&self) -> Vec<Temp> {
		vec![&self.length].into_iter().flat_map(|v| v.unwrap_temp()).collect()
	}
	fn get_write(&self) -> Option<Temp> {
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
	fn get_alloc(&self) -> Option<(Temp, Value)> {
		Some((self.target.clone(), self.length.clone()))
	}
	fn replace_read(&mut self, old: Temp, new: Value) {
		if self.length == Value::Temp(old.clone()) {
			self.length = new;
		}
	}

	fn map_temp(&mut self, _map: &HashMap<Temp, Value>) {
		//noting to do
	}

	fn replaceable(&self, _map: &HashMap<Temp, Value>) -> bool {
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

impl UseTemp<Temp> for StoreInstr {
	fn get_read(&self) -> Vec<Temp> {
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
	fn replace_read(&mut self, old: Temp, new: Value) {
		if self.value == Value::Temp(old.clone()) {
			self.value = new.clone();
		}
		if self.addr == Value::Temp(old.clone()) {
			self.addr = new;
		}
	}

	fn map_temp(&mut self, map: &HashMap<Temp, Value>) {
		map_llvm_temp(&mut self.value, map);
		map_llvm_temp(&mut self.addr, map);
	}

	fn replaceable(&self, _map: &HashMap<Temp, Value>) -> bool {
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

impl UseTemp<Temp> for LoadInstr {
	fn get_read(&self) -> Vec<Temp> {
		vec![&self.addr].into_iter().flat_map(|v| v.unwrap_temp()).collect()
	}
	fn get_write(&self) -> Option<Temp> {
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
	fn replace_read(&mut self, old: Temp, new: Value) {
		if self.addr == Value::Temp(old.clone()) {
			self.addr = new;
		}
	}

	fn map_temp(&mut self, map: &HashMap<Temp, Value>) {
		map_llvm_temp(&mut self.addr, map);
	}

	fn replaceable(&self, _map: &HashMap<Temp, Value>) -> bool {
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

impl UseTemp<Temp> for GEPInstr {
	fn get_read(&self) -> Vec<Temp> {
		unwrap_values(vec![&self.addr, &self.offset])
	}
	fn get_write(&self) -> Option<Temp> {
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
	fn replace_read(&mut self, old: Temp, new: Value) {
		if self.addr == Value::Temp(old.clone()) {
			self.addr = new.clone();
		}
		if self.offset == Value::Temp(old.clone()) {
			self.offset = new;
		}
	}

	fn map_temp(&mut self, map: &HashMap<Temp, Value>) {
		map_llvm_temp(&mut self.addr, map);
		map_llvm_temp(&mut self.offset, map);
	}

	fn replaceable(&self, _map: &HashMap<Temp, Value>) -> bool {
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

impl UseTemp<Temp> for CallInstr {
	fn get_read(&self) -> Vec<Temp> {
		unwrap_values(self.params.iter().map(|(_, x)| x).collect())
	}
	fn get_write(&self) -> Option<Temp> {
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
	fn replace_read(&mut self, old: Temp, new: Value) {
		for (_, v) in self.params.iter_mut() {
			if v == &Value::Temp(old.clone()) {
				*v = new.clone();
			}
		}
	}

	fn map_temp(&mut self, map: &HashMap<Temp, Value>) {
		for (_vartype, value) in &mut self.params {
			map_llvm_temp(value, map);
		}
	}
	fn replaceable(&self, _map: &HashMap<Temp, Value>) -> bool {
		false
	}
}
