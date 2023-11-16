pub mod calc;
pub mod impls;
pub mod utils;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug)]
pub enum BType {
	Int,
	Float,
}

#[derive(Clone, Copy, Debug)]
pub enum FuncRetType {
	Int,
	Float,
	Void,
}

pub type VarType = (bool, BType, Vec<usize>);
pub type FuncType = (FuncRetType, Vec<VarType>);
pub type IntPtr = (Vec<usize>, Array<i32>);
pub type FloatPtr = (Vec<usize>, Array<f32>);

#[derive(Clone, Debug)]
pub enum Value {
	Int(i32),
	Float(f32),
	IntPtr(IntPtr),
	FloatPtr(FloatPtr),
}

pub type Array<T> = (usize, HashMap<Vec<usize>, T>);

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
pub enum UnaryOp {
	Plus,
	Neg,
	Not,
}
