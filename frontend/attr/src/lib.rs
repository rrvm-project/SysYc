mod impls;

use rrvm_symbol::{FuncSymbol, VarSymbol};
use utils::InitValueItem;
use value::{Value, VarType};

#[derive(Clone, Debug)]
pub enum Attr {
	FuncSymbol(FuncSymbol),
	VarSymbol(VarSymbol),
	VarType(VarType),
	Value(Value),
	IRValue(llvm::llvmop::Value),
	InitListHeight(usize),
	InitListPosition(usize),
	GlobalValue(Vec<InitValueItem>),
}

pub trait Attrs {
	fn set_attr(&mut self, name: &str, attr: Attr);
	fn get_attr(&self, name: &str) -> Option<&Attr>;
	fn get_name(&self) -> String;
}
