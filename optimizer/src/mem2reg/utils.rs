use std::{
	collections::{HashMap, HashSet},
	hash::Hash,
};

use llvm::{LlvmTemp, Value, VarType};

use crate::number::Number;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Addr {
	pub base: Number,
	pub offset: Number,
}

impl Addr {
	pub fn new(base: Number, offset: Number) -> Self {
		Addr { base, offset }
	}
}

#[derive(Default, Clone)]
pub struct ArrayInfo {
	pub pos: HashMap<Number, HashMap<Number, HashSet<Number>>>,
}

impl ArrayInfo {
	pub fn insert(&mut self, addr: Addr) {
		let entry = self.pos.entry(addr.base).or_default();
		entry.entry(addr.offset.get_base()).or_default().insert(addr.offset);
	}
	pub fn solve_conflict(&mut self, addr: &Addr) -> Vec<Addr> {
		let entry = self.pos.entry(addr.base.clone()).or_default();
		let base = addr.offset.get_base();
		let mut result = Vec::new();
		entry.retain(|k, set| {
			*k == base || {
				result.extend(set.drain());
				false
			}
		});
		result.into_iter().map(|v| Addr::new(addr.base.clone(), v)).collect()
	}
	pub fn remove(&mut self, base: &Number) -> Vec<Addr> {
		let set = self.pos.remove(base).unwrap_or_default();
		set
			.into_values()
			.flat_map(|v| v.into_iter().map(|v| Addr::new(base.clone(), v)))
			.collect()
	}
}

#[derive(Clone)]
pub enum MemItem {
	Value(Value),
	PhiDef(i32),
}

impl From<Value> for MemItem {
	fn from(value: Value) -> Self {
		MemItem::Value(value)
	}
}

impl From<i32> for MemItem {
	fn from(value: i32) -> Self {
		MemItem::PhiDef(value)
	}
}

#[derive(Default, Clone)]
pub struct ArrayState {
	mapper: HashMap<Addr, MemItem>,
	info: ArrayInfo,
	pub temp_mapper: HashMap<LlvmTemp, Value>,
}

impl ArrayState {
	pub fn load(&mut self, addr: Addr, value: Value) {
		self.info.insert(addr.clone());
		self.mapper.insert(addr, value.into());
	}
	pub fn store(&mut self, addr: Addr, value: Value) {
		self.info.insert(addr.clone());
		self.mapper.insert(addr.clone(), value.into());
		let pos = self.info.solve_conflict(&addr);
		pos.iter().for_each(|addr| {
			self.mapper.remove(addr);
		});
	}
	pub fn insert_item(&mut self, addr: Addr, block_id: i32) {
		self.info.insert(addr.clone());
		self.mapper.insert(addr, block_id.into());
	}
	pub fn remove_base(&mut self, base: &Number) {
		let pos = self.info.remove(base);
		pos.iter().for_each(|addr| {
			self.mapper.remove(addr);
		});
	}
	pub fn get(&self, addr: &Addr) -> Option<&MemItem> {
		self.mapper.get(addr)
	}
	pub fn insert(&mut self, temp: LlvmTemp, value: Value) {
		self.temp_mapper.insert(temp, value);
	}
	pub fn remove(&mut self, addr: &Addr) {
		self.mapper.remove(addr);
	}
	pub fn map_value(&self, value: &Value) -> Value {
		match value {
			Value::Temp(temp) => self.temp_mapper.get(temp).unwrap_or(value),
			_ => value,
		}
		.clone()
	}
}

#[derive(Default, Clone, PartialEq, Eq)]
pub struct UseState {
	pub loads: HashSet<Addr>,
	pub stores: HashSet<Addr>,
}

#[derive(Default, Clone, PartialEq, Eq)]
pub struct UseStateItem {
	pub state_in: UseState,
	pub state_out: UseState,
}

impl UseState {}

#[derive(Default)]
pub struct AddrInfo {
	pub defs: HashSet<i32>,
	pub var_type: VarType,
}

impl AddrInfo {
	pub fn insert_def(&mut self, def: i32) {
		self.defs.insert(def);
	}
}
