use std::collections::HashMap;

use llvm::{LlvmTemp, Value};

use crate::number::Number;

#[derive(Default)]
pub struct FuncData {
	pub num_mapper: HashMap<LlvmTemp, Number>,
}

impl FuncData {
	pub fn clear_num_mapper(&mut self) {
		self.num_mapper.clear();
	}
	pub fn set_number(&mut self, temp: LlvmTemp, number: Number) {
		self.num_mapper.insert(temp, number);
	}
	pub fn get_number(&self, temp: &LlvmTemp) -> Option<&Number> {
		self.num_mapper.get(temp)
	}
	pub fn get_val_number(&self, value: &Value) -> Option<Number> {
		match value {
			Value::Int(val) => Some(Number::from(*val as u32)),
			Value::Float(val) => Some(Number::from(val.to_bits())),
			Value::Temp(temp) => self.get_number(temp).cloned(),
		}
	}
}

#[derive(Default)]
pub struct MetaData {
	pub func_data: HashMap<String, FuncData>,
}

impl MetaData {
	pub fn new() -> Self {
		Self {
			func_data: HashMap::new(),
		}
	}
	pub fn get_func_data(&mut self, func_name: &str) -> &mut FuncData {
		self.func_data.entry(func_name.to_string()).or_default()
	}
}