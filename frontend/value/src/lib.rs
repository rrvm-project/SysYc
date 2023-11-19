pub mod calc;
pub mod impls;
pub mod utils;
use std::collections::HashMap;

use llvm::llvmvar;

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
// isconst, btype, dim_list
pub type VarType = (bool, BType, Vec<usize>);
pub type FuncType = (FuncRetType, Vec<VarType>);
pub type IntPtr = (Vec<usize>, Array<i32>);
pub type FloatPtr = (Vec<usize>, Array<f32>);

pub fn to_llvm_var_type(t: &VarType) -> llvmvar::VarType {
	match (t.1, t.2.len()) {
		(BType::Int, 0) => llvmvar::VarType::I32,
		(BType::Int, _) => llvmvar::VarType::I32Ptr,
		(BType::Float, 0) => llvmvar::VarType::F32,
		(BType::Float, _) => llvmvar::VarType::F32Ptr,
	}
}

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
