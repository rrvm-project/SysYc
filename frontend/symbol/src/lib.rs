pub mod impls;
pub mod manager;

#[derive(Clone)]
pub struct Symbol<T> {
	pub id: i32,
	pub ident: String,
	pub var_type: T,
}

#[derive(Clone)]
pub enum BType {
	Int,
	Float,
}

pub type VarType = (BType, Vec<usize>);
pub type FuncType = Vec<VarType>;

pub type VarSymbol = Symbol<VarType>;
pub type FuncSymbol = Symbol<FuncType>;
