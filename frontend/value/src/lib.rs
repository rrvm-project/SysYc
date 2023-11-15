pub mod calc;
pub mod impls;
pub mod utils;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum BType {
	Int,
	Float,
}

pub type VarType = (bool, BType, Vec<usize>);
pub type FuncType = Vec<VarType>;

#[derive(Clone, Debug)]
pub enum Value {
	Int(i32),
	Float(f32),
	IntPtr(Vec<usize>, Array<i32>),
	FloatPtr(Vec<usize>, Array<f32>),
}

pub type Array<T> = (usize, HashMap<Vec<usize>, T>);

#[derive(Debug, PartialEq, Eq)]
pub enum BinaryOp {
	Assign,
	Add,
	Sub,
	Mul,
	Div,
	Mod,
	LT,
	LE,
	GE,
	GT,
	EQ,
	NE,
	IDX,
}

#[derive(Debug)]
pub enum UnaryOp {
	Plus,
	Neg,
	Not,
}
