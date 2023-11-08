// TODO: 另起一个文件用来描述symbol
use ir_type::builtin_type::{BaseType, IRType};

#[derive(Debug, Clone)]
pub struct VarSymbol {
	pub name: String,
	pub tp: IRType,
	pub is_global: bool,
	pub id: usize,
}

#[derive(Debug, Clone)]
pub struct FuncSymbol {
	pub name: String,
	pub ret_t: BaseType,
	pub params: Vec<VarSymbol>,
	pub id: usize,
}

#[allow(dead_code)]
impl FuncSymbol {
	pub fn add_param(&mut self, param: &VarSymbol) {
		self.params.push(param.clone());
	}
	pub fn param_num(&self) -> usize {
		self.params.len()
	}
	pub fn get_param(&self, index: usize) -> Option<&VarSymbol> {
		self.params.get(index)
	}
	pub fn get_all_params(&self) -> &Vec<VarSymbol> {
		&self.params
	}
}
