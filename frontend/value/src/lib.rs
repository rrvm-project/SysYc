pub mod calc;
pub mod calc_type;
pub mod impls;
pub mod utils;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
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

#[derive(Clone, Debug)]
pub struct VarType {
	pub is_lval: bool,
	pub type_t: BType,
	pub dims: Vec<usize>,
}

pub type FuncType = (FuncRetType, Vec<VarType>);
pub type IntPtr = (Vec<usize>, Vec<i32>);
pub type FloatPtr = (Vec<usize>, Vec<f32>);

#[derive(Clone, Debug)]
pub enum Value {
	Int(i32),
	Float(f32),
	IntPtr(IntPtr),
	FloatPtr(FloatPtr),
}

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
