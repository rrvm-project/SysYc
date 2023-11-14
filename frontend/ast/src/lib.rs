pub mod impls;
pub mod tree;
pub mod visitor;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VarType {
	Int,
	Float,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FuncType {
	Int,
	Float,
	Void,
}

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
