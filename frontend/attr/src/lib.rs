mod impls;

use rrvm_symbol::{FuncSymbol, VarSymbol};
use value::{Value, VarType};

#[derive(Clone, Debug)]
pub enum Attr {
	FuncSymbol(FuncSymbol),
	VarSymbol(VarSymbol),
	VarType(VarType),
	Value(Value),
}

pub trait Attrs {
	fn set_attr(&mut self, name: &str, attr: Attr);
	fn get_attr(&self, name: &str) -> Option<&Attr>;
}
