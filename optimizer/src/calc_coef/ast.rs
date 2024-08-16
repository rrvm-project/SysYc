use std::{cell::RefCell, rc::Rc};

use llvm::{Value, VarType};
// use utils::Label;
// pub enum AstNode {
// 	Value(Value),
// 	Expr((Rc<RefCell<AstNode>>, Box<dyn LlvmOp>, Rc<RefCell<AstNode>>)),
// 	CallVal(String, Vec<Rc<RefCell<AstNode>>>),
// 	PhiNode(Vec<(Rc<RefCell<AstNode>>, Label)>),
// 	ConvertNode(VarType, Rc<RefCell<AstNode>>),
// 	GepNode(Rc<RefCell<AstNode>>, Rc<RefCell<AstNode>>),
// 	AllocNode(Value),
// }
