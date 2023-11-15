pub mod impls;
pub mod tree;
pub mod visitor;

pub use visitor::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VarType {
	Int,
	Float,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FuncRetType {
	Int,
	Float,
	Void,
}
