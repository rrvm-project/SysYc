pub mod tree;
pub mod visitor;

#[derive(Debug)]
pub enum Type {
	Int,
	Float,
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
