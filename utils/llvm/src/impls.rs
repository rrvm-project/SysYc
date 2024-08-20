#![allow(clippy::new_ret_no_self)]

use std::fmt::Display;

use utils::{Label, UseTemp};

use std::collections::HashMap;

use crate::{
	llvminstr::*, llvmop::*, utils::*, LlvmInstrVariant, LlvmTemp, VarType,
};

impl<T: Into<Value> + Clone> From<&T> for Value {
	fn from(value: &T) -> Self {
		value.clone().into()
	}
}

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

fn map_llvm_temp_to_value(temp: &mut Value, map: &HashMap<LlvmTemp, Value>) {
	if let Value::Temp(v) = temp {
		if let Some(v) = map.get(v) {
			*temp = v.clone();
		}
	}
}

fn map_llvm_temp_to_temp(
	temp: &mut LlvmTemp,
	map: &HashMap<LlvmTemp, LlvmTemp>,
) {
	if let Some(v) = map.get(temp) {
		*temp = v.clone();
	}
}

impl LlvmInstrTrait for ArithInstr {
	fn get_variant(&self) -> LlvmInstrVariant {
		LlvmInstrVariant::ArithInstr(self)
	}
	fn map_temp(&mut self, map: &HashMap<LlvmTemp, Value>) {
		map_llvm_temp_to_value(&mut self.lhs, map);
		map_llvm_temp_to_value(&mut self.rhs, map);
	}
	fn map_all_temp(&mut self, map: &HashMap<LlvmTemp, LlvmTemp>) {
		if let Value::Temp(t) = &mut self.lhs {
			map_llvm_temp_to_temp(t, map);
		}
		if let Value::Temp(t) = &mut self.rhs {
			map_llvm_temp_to_temp(t, map);
		}
		map_llvm_temp_to_temp(&mut self.target, map);
	}
	fn set_target(&mut self, target: LlvmTemp) {
		self.target = target
	}
	fn replaceable(&self, map: &HashMap<LlvmTemp, Value>) -> bool {
		map.get(&self.target).is_some()
	}
	// 在强度削弱时使用，用于判断候选操作
	fn get_candidate_operator(&self) -> Option<ArithOp> {
		match self.op {
			ArithOp::Add
			| ArithOp::Fadd
			| ArithOp::Mul
			| ArithOp::Fmul
			| ArithOp::Sub
			| ArithOp::Fsub
			| ArithOp::Div
			| ArithOp::Fdiv
			| ArithOp::Rem => Some(self.op),
			_ => None,
		}
	}
	fn get_lhs_and_rhs(&self) -> Option<(Value, Value)> {
		Some((self.lhs.clone(), self.rhs.clone()))
	}
	fn get_read_values(&self) -> Vec<Value> {
		vec![self.lhs.clone(), self.rhs.clone()]
	}
	fn set_read_values(&mut self, id: usize, value: Value) {
		match id {
			0 => self.lhs = value,
			1 => self.rhs = value,
			_ => unreachable!(),
		}
	}
}

impl ArithInstr {
	pub fn new(
		target: LlvmTemp,
		lhs: impl Into<Value>,
		op: ArithOp,
		rhs: impl Into<Value>,
		var_type: VarType,
	) -> LlvmInstr {
		Box::new(Self {
			target,
			lhs: lhs.into(),
			op,
			rhs: rhs.into(),
			var_type,
		})
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
	fn map_temp(&mut self, map: &HashMap<LlvmTemp, Value>) {
		map_llvm_temp_to_value(&mut self.lhs, map);
		map_llvm_temp_to_value(&mut self.rhs, map);
	}
	fn map_all_temp(&mut self, map: &HashMap<LlvmTemp, LlvmTemp>) {
		if let Value::Temp(t) = &mut self.lhs {
			map_llvm_temp_to_temp(t, map);
		}
		if let Value::Temp(t) = &mut self.rhs {
			map_llvm_temp_to_temp(t, map);
		}
		map_llvm_temp_to_temp(&mut self.target, map);
	}
	fn set_target(&mut self, target: LlvmTemp) {
		self.target = target
	}
	fn replaceable(&self, map: &HashMap<LlvmTemp, Value>) -> bool {
		map.get(&self.target).is_some()
	}
	fn get_read_values(&self) -> Vec<Value> {
		vec![self.lhs.clone(), self.rhs.clone()]
	}
	fn set_read_values(&mut self, id: usize, value: Value) {
		match id {
			0 => self.lhs = value,
			1 => self.rhs = value,
			_ => unreachable!(),
		}
	}
	fn is_cmp(&self) -> bool {
		true
	}
	fn get_lhs_and_rhs(&self) -> Option<(Value, Value)> {
		Some((self.lhs.clone(), self.rhs.clone()))
	}
}

impl ConvertInstr {
	pub fn new(
		target: LlvmTemp,
		lhs: impl Into<Value>,
		op: ConvertOp,
		var_type: VarType,
	) -> LlvmInstr {
		Box::new(Self {
			target,
			lhs: lhs.into(),
			op,
			var_type,
		})
	}
}
impl Display for ConvertInstr {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(
			f,
			"  {} {} = {} {}",
			self.target, self.var_type, self.op, self.lhs
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
	fn map_temp(&mut self, map: &HashMap<LlvmTemp, Value>) {
		map_llvm_temp_to_value(&mut self.lhs, map);
	}
	fn map_all_temp(&mut self, map: &HashMap<LlvmTemp, LlvmTemp>) {
		if let Value::Temp(t) = &mut self.lhs {
			map_llvm_temp_to_temp(t, map);
		}
		map_llvm_temp_to_temp(&mut self.target, map);
	}
	fn set_target(&mut self, target: LlvmTemp) {
		self.target = target
	}
	fn replaceable(&self, map: &HashMap<LlvmTemp, Value>) -> bool {
		map.get(&self.target).is_some()
	}
	fn get_read_values(&self) -> Vec<Value> {
		vec![self.lhs.clone()]
	}
	fn set_read_values(&mut self, id: usize, value: Value) {
		match id {
			0 => self.lhs = value,
			_ => unreachable!(),
		}
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
	fn get_read_values(&self) -> Vec<Value> {
		Vec::new()
	}
	fn set_read_values(&mut self, _id: usize, _value: Value) {
		unreachable!("JumpInstr has no read values")
	}
	fn get_label(&self) -> Label {
		self.target.clone()
	}
	fn map_label(&mut self, map: &HashMap<Label, Label>) {
		if let Some(new_label) = map.get(&self.target) {
			self.target = new_label.clone()
		}
	}
}

impl JumpInstr {
	pub fn new(target: Label) -> LlvmInstr {
		Box::new(Self { target })
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
	fn map_label(&mut self, map: &HashMap<Label, Label>) {
		if let Some(new_label) = map.get(&self.target_true) {
			self.target_true = new_label.clone();
		}
		if let Some(new_label) = map.get(&self.target_false) {
			self.target_false = new_label.clone();
		}
	}
	fn map_temp(&mut self, map: &HashMap<LlvmTemp, Value>) {
		map_llvm_temp_to_value(&mut self.cond, map);
	}
	fn map_all_temp(&mut self, map: &HashMap<LlvmTemp, LlvmTemp>) {
		if let Value::Temp(t) = &mut self.cond {
			map_llvm_temp_to_temp(t, map);
		}
	}
	fn get_read_values(&self) -> Vec<Value> {
		vec![self.cond.clone()]
	}
	fn set_read_values(&mut self, id: usize, value: Value) {
		match id {
			0 => self.cond = value,
			_ => unreachable!(),
		}
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
	fn is_phi(&self) -> bool {
		true
	}
	fn map_temp(&mut self, map: &HashMap<LlvmTemp, Value>) {
		for (value, _) in &mut self.source {
			map_llvm_temp_to_value(value, map);
		}
	}
	fn map_all_temp(&mut self, map: &HashMap<LlvmTemp, LlvmTemp>) {
		for (value, _) in &mut self.source {
			if let Value::Temp(t) = value {
				map_llvm_temp_to_temp(t, map);
			}
		}
		map_llvm_temp_to_temp(&mut self.target, map);
	}
	fn set_target(&mut self, target: LlvmTemp) {
		self.target = target
	}
	fn map_label(&mut self, map: &HashMap<Label, Label>) {
		for (_, label) in self.source.iter_mut() {
			if let Some(new_label) = map.get(label) {
				*label = new_label.clone();
			}
		}
	}
	fn get_read_values(&self) -> Vec<Value> {
		self.source.iter().map(|(v, _)| v.clone()).collect()
	}
	fn set_read_values(&mut self, id: usize, value: Value) {
		assert!(
			id < self.source.len(),
			"id of read values out of range for phi"
		);
		self.source[id].0 = value;
	}
}

impl PhiInstr {
	pub fn new(target: LlvmTemp, source: Vec<(Value, Label)>) -> Self {
		Self {
			var_type: target.var_type,
			target,
			source,
		}
	}
	pub fn get_value(&self, label: &Label) -> Option<Value> {
		self
			.source
			.iter()
			.find_map(|(v, l)| if l == label { Some(v.clone()) } else { None })
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
			map_llvm_temp_to_value(value, map);
		}
	}
	fn map_all_temp(&mut self, map: &HashMap<LlvmTemp, LlvmTemp>) {
		if let Some(Value::Temp(t)) = &mut self.value {
			map_llvm_temp_to_temp(t, map);
		}
	}
	fn map_label(&mut self, _map: &HashMap<Label, Label>) {}
	fn get_read_values(&self) -> Vec<Value> {
		self.value.as_ref().map_or(Vec::new(), |v| vec![v.clone()])
	}
	fn set_read_values(&mut self, id: usize, value: Value) {
		if let Some(old) = self.value.as_mut() {
			assert_eq!(id, 0, "id of read values out of range for ret");
			*old = value;
		} else {
			unreachable!("set read values for ret void");
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
	fn get_alloc(&self) -> Option<(LlvmTemp, Value)> {
		Some((self.target.clone(), self.length.clone()))
	}
	fn map_temp(&mut self, map: &HashMap<LlvmTemp, Value>) {
		map_llvm_temp_to_value(&mut self.length, map);
	}
	fn map_all_temp(&mut self, map: &HashMap<LlvmTemp, LlvmTemp>) {
		if let Value::Temp(t) = &mut self.length {
			map_llvm_temp_to_temp(t, map);
		}
		map_llvm_temp_to_temp(&mut self.target, map);
	}
	fn set_target(&mut self, target: LlvmTemp) {
		self.target = target
	}
	fn get_read_values(&self) -> Vec<Value> {
		vec![self.length.clone()]
	}
	fn set_read_values(&mut self, id: usize, value: Value) {
		match id {
			0 => self.length = value,
			_ => unreachable!("invalid id of read values for alloc"),
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

impl UseTemp<LlvmTemp> for StoreInstr {
	fn get_read(&self) -> Vec<LlvmTemp> {
		unwrap_values(vec![&self.value, &self.addr])
	}
}

impl LlvmInstrTrait for StoreInstr {
	fn get_variant(&self) -> LlvmInstrVariant {
		LlvmInstrVariant::StoreInstr(self)
	}
	fn has_sideeffect(&self) -> bool {
		true
	}
	fn is_store(&self) -> bool {
		true
	}
	fn map_temp(&mut self, map: &HashMap<LlvmTemp, Value>) {
		map_llvm_temp_to_value(&mut self.value, map);
		map_llvm_temp_to_value(&mut self.addr, map);
	}
	fn map_all_temp(&mut self, map: &HashMap<LlvmTemp, LlvmTemp>) {
		if let Value::Temp(t) = &mut self.value {
			map_llvm_temp_to_temp(t, map);
		}
		if let Value::Temp(t) = &mut self.addr {
			map_llvm_temp_to_temp(t, map);
		}
	}
	fn get_read_values(&self) -> Vec<Value> {
		vec![self.value.clone(), self.addr.clone()]
	}
	fn set_read_values(&mut self, id: usize, value: Value) {
		match id {
			0 => self.value = value,
			1 => self.addr = value,
			_ => unreachable!("invalid id of read values for store"),
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
	fn is_load(&self) -> bool {
		self.addr.unwrap_temp().map_or(true, |v| !v.is_global)
	}
	fn map_temp(&mut self, map: &HashMap<LlvmTemp, Value>) {
		map_llvm_temp_to_value(&mut self.addr, map);
	}
	fn map_all_temp(&mut self, map: &HashMap<LlvmTemp, LlvmTemp>) {
		if let Value::Temp(t) = &mut self.addr {
			map_llvm_temp_to_temp(t, map);
		}
		map_llvm_temp_to_temp(&mut self.target, map);
	}
	fn set_target(&mut self, target: LlvmTemp) {
		self.target = target
	}
	fn get_read_values(&self) -> Vec<Value> {
		vec![self.addr.clone()]
	}
	fn set_read_values(&mut self, id: usize, value: Value) {
		match id {
			0 => self.addr = value,
			_ => unreachable!("invalid id of read values for load"),
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
	fn map_temp(&mut self, map: &HashMap<LlvmTemp, Value>) {
		map_llvm_temp_to_value(&mut self.addr, map);
		map_llvm_temp_to_value(&mut self.offset, map);
	}
	fn map_all_temp(&mut self, map: &HashMap<LlvmTemp, LlvmTemp>) {
		if let Value::Temp(t) = &mut self.addr {
			map_llvm_temp_to_temp(t, map);
		}
		if let Value::Temp(t) = &mut self.offset {
			map_llvm_temp_to_temp(t, map);
		}
		map_llvm_temp_to_temp(&mut self.target, map);
	}
	fn set_target(&mut self, target: LlvmTemp) {
		self.target = target
	}
	fn get_read_values(&self) -> Vec<Value> {
		vec![self.addr.clone(), self.offset.clone()]
	}
	fn set_read_values(&mut self, id: usize, value: Value) {
		match id {
			0 => self.addr = value,
			1 => self.offset = value,
			_ => unreachable!("invalid id of read values for gep"),
		}
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
	fn get_label(&self) -> Label {
		self.func.clone()
	}
	fn map_temp(&mut self, map: &HashMap<LlvmTemp, Value>) {
		for (_, value) in &mut self.params {
			map_llvm_temp_to_value(value, map);
		}
	}
	fn map_all_temp(&mut self, map: &HashMap<LlvmTemp, LlvmTemp>) {
		for (_, value) in &mut self.params {
			if let Value::Temp(t) = value {
				map_llvm_temp_to_temp(t, map);
			}
		}
		map_llvm_temp_to_temp(&mut self.target, map);
	}
	fn set_target(&mut self, target: LlvmTemp) {
		self.target = target
	}
	fn get_read_values(&self) -> Vec<Value> {
		self.params.iter().map(|(_, x)| x.clone()).collect()
	}
	fn set_read_values(&mut self, id: usize, value: Value) {
		assert!(
			id < self.params.len(),
			"id of read values out of range for call"
		);
		self.params[id].1 = value;
	}
	fn replaceable(&self, map: &HashMap<LlvmTemp, Value>) -> bool {
		map.get(&self.target).is_some()
	}
}
