use std::{collections::HashMap, fmt::Display};

use utils::UseTemp;

use crate::{
	llvminstr::*,
	llvminstrattr::{LlvmAttr, LlvmAttrs},
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
				_attrs: HashMap::new(),
				target: self.target_true.clone(),
			});
		}
		if self.cond.always_false() {
			return Some(JumpInstr {
				_attrs: HashMap::new(),
				target: self.target_false.clone(),
			});
		}
		if self.target_true == self.target_false {
			return Some(JumpInstr {
				_attrs: HashMap::new(),
				target: self.target_false.clone(),
			});
		}
		None
	}
	fn is_jump_cond(&self) -> bool {
		true
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
	fn is_store(&self) -> bool {
		true
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
		true
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
	fn is_call(&self) -> bool {
		true
	}
}

impl LlvmAttrs for ArithInstr {
	fn set_attr(&mut self, name: &str, attr: LlvmAttr) {
		self._attrs.insert(String::from(name), attr);
	}

	fn get_attr(&self, name: &str) -> Option<&LlvmAttr> {
		self._attrs.get(name)
	}

	fn clear_attr(&mut self, name: &str) {
		self._attrs.remove(name);
	}
}

impl LlvmAttrs for CompInstr {
	fn set_attr(&mut self, name: &str, attr: LlvmAttr) {
		self._attrs.insert(String::from(name), attr);
	}

	fn get_attr(&self, name: &str) -> Option<&LlvmAttr> {
		self._attrs.get(name)
	}

	fn clear_attr(&mut self, name: &str) {
		self._attrs.remove(name);
	}
}
impl LlvmAttrs for ConvertInstr {
	fn set_attr(&mut self, name: &str, attr: LlvmAttr) {
		self._attrs.insert(String::from(name), attr);
	}

	fn get_attr(&self, name: &str) -> Option<&LlvmAttr> {
		self._attrs.get(name)
	}

	fn clear_attr(&mut self, name: &str) {
		self._attrs.remove(name);
	}
}
impl LlvmAttrs for JumpInstr {
	fn set_attr(&mut self, name: &str, attr: LlvmAttr) {
		self._attrs.insert(String::from(name), attr);
	}

	fn get_attr(&self, name: &str) -> Option<&LlvmAttr> {
		self._attrs.get(name)
	}

	fn clear_attr(&mut self, name: &str) {
		self._attrs.remove(name);
	}
}
impl LlvmAttrs for JumpCondInstr {
	fn set_attr(&mut self, name: &str, attr: LlvmAttr) {
		self._attrs.insert(String::from(name), attr);
	}

	fn get_attr(&self, name: &str) -> Option<&LlvmAttr> {
		self._attrs.get(name)
	}

	fn clear_attr(&mut self, name: &str) {
		self._attrs.remove(name);
	}
}
impl LlvmAttrs for PhiInstr {
	fn set_attr(&mut self, name: &str, attr: LlvmAttr) {
		self._attrs.insert(String::from(name), attr);
	}

	fn get_attr(&self, name: &str) -> Option<&LlvmAttr> {
		self._attrs.get(name)
	}

	fn clear_attr(&mut self, name: &str) {
		self._attrs.remove(name);
	}
}
impl LlvmAttrs for RetInstr {
	fn set_attr(&mut self, name: &str, attr: LlvmAttr) {
		self._attrs.insert(String::from(name), attr);
	}

	fn get_attr(&self, name: &str) -> Option<&LlvmAttr> {
		self._attrs.get(name)
	}

	fn clear_attr(&mut self, name: &str) {
		self._attrs.remove(name);
	}
}
impl LlvmAttrs for AllocInstr {
	fn set_attr(&mut self, name: &str, attr: LlvmAttr) {
		self._attrs.insert(String::from(name), attr);
	}

	fn get_attr(&self, name: &str) -> Option<&LlvmAttr> {
		self._attrs.get(name)
	}

	fn clear_attr(&mut self, name: &str) {
		self._attrs.remove(name);
	}
}
impl LlvmAttrs for LoadInstr {
	fn set_attr(&mut self, name: &str, attr: LlvmAttr) {
		self._attrs.insert(String::from(name), attr);
	}

	fn get_attr(&self, name: &str) -> Option<&LlvmAttr> {
		self._attrs.get(name)
	}

	fn clear_attr(&mut self, name: &str) {
		self._attrs.remove(name);
	}
}
impl LlvmAttrs for StoreInstr {
	fn set_attr(&mut self, name: &str, attr: LlvmAttr) {
		self._attrs.insert(String::from(name), attr);
	}

	fn get_attr(&self, name: &str) -> Option<&LlvmAttr> {
		self._attrs.get(name)
	}

	fn clear_attr(&mut self, name: &str) {
		self._attrs.remove(name);
	}
}
impl LlvmAttrs for GEPInstr {
	fn set_attr(&mut self, name: &str, attr: LlvmAttr) {
		self._attrs.insert(String::from(name), attr);
	}

	fn get_attr(&self, name: &str) -> Option<&LlvmAttr> {
		self._attrs.get(name)
	}

	fn clear_attr(&mut self, name: &str) {
		self._attrs.remove(name);
	}
}
impl LlvmAttrs for CallInstr {
	fn set_attr(&mut self, name: &str, attr: LlvmAttr) {
		self._attrs.insert(String::from(name), attr);
	}

	fn get_attr(&self, name: &str) -> Option<&LlvmAttr> {
		self._attrs.get(name)
	}

	fn clear_attr(&mut self, name: &str) {
		self._attrs.remove(name);
	}
}
