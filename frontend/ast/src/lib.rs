pub mod impls;
pub mod tree;
pub mod visitor;

#[derive(Debug)]
pub enum VarType {
	Int,
	Float,
}

#[derive(Debug)]
pub enum FuncType {
	Int,
	Float,
	Void,
}

#[derive(Debug)]
pub enum BinaryOp {
	Assign,
	Add,
	Sub,
	Mul,
	Div,
	Mod,
	LQ,
	LE,
	GE,
	GQ,
	EQ,
	NE,
}

#[derive(Debug)]
pub enum UnaryOp {
	Plus,
	Neg,
	Not,
}
