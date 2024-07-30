use std::{cell::RefCell, rc::Rc};

use llvm::Value;

#[allow(unused)]
pub struct IndVar {
	base: Value,
	op: IndVarOp,
	step: Option<Rc<RefCell<IndVar>>>,
	is_zfp: Option<Value>,
}

#[allow(unused)]
pub enum IndVarOp {
	Add,
	Fadd,
	Sub,
	Fsub,
	Mul,
	Fmul,
	Div,
	Fdiv,
}
