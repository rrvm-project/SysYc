// TODO: 另起一个文件用来描述symbol
use attr::CompileConstValue;
use ir_type::builtin_type::IRType;
use llvm::temp::Temp;

#[derive(Debug, Clone)]
pub struct VarSymbol {
	pub name: String,
	pub tp: IRType,
	pub is_global: bool,
	pub id: usize,
	pub const_or_global_initial_value: Option<CompileConstValue>,
	pub temp: Option<Temp>,
}

#[derive(Debug, Clone)]
pub struct FuncSymbol {
	pub name: String,
	pub ret_t: IRType,
	pub params: Vec<IRType>,
	pub id: usize,
}
