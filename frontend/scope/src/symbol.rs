// TODO: 另起一个文件用来描述symbol
use ast::{FuncType, VarType};

#[derive(Debug, Clone)]
pub struct VarSymbol {
    pub name: String,
    pub tp: VarType,
    pub is_global: bool,
}

#[derive(Debug, Clone)]
pub struct FuncSymbol {
    pub name: String,
    pub ret_t: FuncType,
    // 这里放VarSymbol还是放VarType?
    pub params: Vec<VarSymbol>,
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