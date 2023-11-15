use value::{FuncType, VarType};

pub mod impls;
pub mod manager;

#[derive(Clone)]
pub struct Symbol<T> {
	pub id: i32,
	pub ident: String,
	pub var_type: T,
}
pub type VarSymbol = Symbol<VarType>;
pub type FuncSymbol = Symbol<FuncType>;
