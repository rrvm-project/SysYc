pub mod parser;

#[derive(Debug, Clone)]
pub enum Val {
	IntLit(i32),
	Int(String),
	FloatLit(f32),
	Float(String),
	Label(String),
	Ptr(String),
	Void,
}
#[derive(Debug, Clone)]
pub enum Type {
	Int,
	Float,
	Ptr,
	Void,
	Label,
}
#[derive(Debug, Clone)]
pub enum IcmpCond {
	Eq,
	Ne,
	Sgt,
	Sge,
	Slt,
	Sle,
}

#[derive(Debug, Clone)]
pub enum LlvmOp {
	Ret(Option<Val>),
	Br(Val, Option<(Val, Val)>),
	Add(Val, Val, Val),
	Sub(Val, Val, Val),
	Mul(Val, Val, Val),
	Div(Val, Val, Val),
	Rem(Val, Val, Val),
	Shl(Val, Val, Val),
	LShr(Val, Val, Val),
	AShr(Val, Val, Val),
	And(Val, Val, Val),
	Or(Val, Val, Val),
	Xor(Val, Val, Val),
	Icmp(Val, IcmpCond, Val, Val),
	//TODO:Discuss whether to implement float comparison as Icmp or Fcmp
	Phi(Val, Vec<(Val, Val)>),
	Alloca(Val, Val),
	Store(Val, Val),
	Load(Val, Val),
	Call(Val, String, Vec<Val>, bool), //设置bool是方便尾调用优化
	GloblVar(Val, Val),
	FuncDecl(Type, String, Vec<Val>),
	Label(Val),
}
