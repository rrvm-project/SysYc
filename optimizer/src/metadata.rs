use std::collections::{HashMap, HashSet};

use llvm::{LlvmTemp, Value};

use crate::number::Number;

/// Identifier of global variable (as long to func params)
pub type VarIdent = (String, usize);

#[derive(Default, Debug, Clone, Copy)]
pub struct VarData {
	pub to_load: bool,
	pub to_store: bool,
}

#[derive(Default, Clone, Debug)]
pub struct UsageInfo {
	pub may_loads: HashSet<VarIdent>,
	pub may_stores: HashSet<VarIdent>,
}

impl UsageInfo {
	pub fn clear(&mut self) {
		self.may_loads.clear();
		self.may_stores.clear();
	}
}

#[derive(Default)]
pub struct FuncData {
	pub num_mapper: HashMap<LlvmTemp, Number>,
	pub pure: bool,
	pub usage_info: UsageInfo,
}

impl FuncData {
	pub fn clear_num_mapper(&mut self) {
		self.num_mapper.clear();
	}
	pub fn clear_usage_info(&mut self) {
		self.usage_info.clear();
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
	pub fn may_load(&self, global_var: &VarIdent) -> bool {
		self.usage_info.may_loads.contains(global_var)
	}
	pub fn may_store(&self, global_var: &VarIdent) -> bool {
		self.usage_info.may_stores.contains(global_var)
	}
	pub fn set_may_load(&mut self, global_var: VarIdent) {
		self.usage_info.may_loads.insert(global_var);
	}
	pub fn set_may_store(&mut self, global_var: VarIdent) {
		self.usage_info.may_stores.insert(global_var);
	}
	pub fn set_syscall(&mut self) {
		self.pure = false;
		self.usage_info.may_loads.insert(("系统调用".to_owned(), 0));
		self.usage_info.may_stores.insert(("系统调用".to_owned(), 0));
	}
}

#[derive(Default)]
pub struct MetaData {
	pub func_data: HashMap<String, FuncData>,
	pub var_data: HashMap<VarIdent, VarData>,
}

impl MetaData {
	pub fn new() -> Self {
		Self {
			func_data: HashMap::new(),
			var_data: HashMap::new(),
		}
	}
	pub fn get_func_data(&mut self, func_name: &str) -> &mut FuncData {
		self.func_data.entry(func_name.to_string()).or_default()
	}
	pub fn is_pure(&mut self, func_name: &str) -> bool {
		self.func_data.get(func_name).map(|data| data.pure).unwrap_or(false)
	}
	pub fn get_var_data(&mut self, var_ident: &VarIdent) -> &mut VarData {
		self.var_data.entry(var_ident.clone()).or_default()
	}
	pub fn may_load(&mut self, func_name: &str, var_ident: &VarIdent) -> bool {
		self
			.func_data
			.get(func_name)
			.map(|data| data.may_load(var_ident))
			.unwrap_or(false)
	}
	pub fn may_store(&mut self, func_name: &str, var_ident: &VarIdent) -> bool {
		self
			.func_data
			.get(func_name)
			.map(|data| data.may_store(var_ident))
			.unwrap_or(false)
	}
}
