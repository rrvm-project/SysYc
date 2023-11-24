use attr::{Attr, Attrs};
use std::fmt::Debug;
use sysyc_derive::{has_attrs, AstNode};
use utils::errors::Result;
use value::{BType, BinaryOp, FuncRetType, UnaryOp};

use crate::visitor::Visitor;

pub trait AstNode: Debug + Attrs {
	fn accept(&mut self, visitor: &mut dyn Visitor) -> Result<()>;
}

pub type Node = Box<dyn AstNode>;
pub type NodeList = Vec<Node>;

#[has_attrs]
#[derive(Debug, AstNode)]
pub struct Program {
	pub comp_units: NodeList,
}

#[has_attrs]
#[derive(Debug, AstNode)]
pub struct VarDef {
	pub ident: String,
	pub dim_list: NodeList,
	pub init: Option<Node>,
}

#[has_attrs]
#[derive(Debug, AstNode)]
pub struct VarDecl {
	pub is_const: bool,
	pub type_t: BType,
	pub defs: NodeList,
}

#[has_attrs]
#[derive(Debug, AstNode)]
pub struct InitValList {
	pub val_list: NodeList,
}

#[has_attrs]
#[derive(Debug, AstNode)]
pub struct LiteralInt {
	pub value: i32,
}

#[has_attrs]
#[derive(Debug, AstNode)]
pub struct LiteralFloat {
	pub value: f32,
}

#[has_attrs]
#[derive(Debug, AstNode)]
pub struct Variable {
	pub ident: String,
}

#[has_attrs]
#[derive(Debug, AstNode)]
pub struct BinaryExpr {
	pub lhs: Node,
	pub op: BinaryOp,
	pub rhs: Node,
}

#[has_attrs]
#[derive(Debug, AstNode)]
pub struct UnaryExpr {
	pub op: UnaryOp,
	pub rhs: Node,
}

#[has_attrs]
#[derive(Debug, AstNode)]
pub struct FuncCall {
	pub ident: String,
	pub params: NodeList,
}

#[has_attrs]
#[derive(Debug, AstNode)]
pub struct FuncDecl {
	pub ret_type: FuncRetType,
	pub ident: String,
	pub formal_params: NodeList,
	pub block: Node,
}

#[has_attrs]
#[derive(Debug, AstNode)]
pub struct FormalParam {
	pub type_t: BType,
	pub ident: String,
	pub dim_list: NodeList,
}

#[has_attrs]
#[derive(Debug, AstNode)]
pub struct Block {
	pub stmts: NodeList,
}

#[has_attrs]
#[derive(Debug, AstNode)]
pub struct If {
	pub cond: Node,
	pub body: Node,
	pub then: Option<Node>,
}

#[has_attrs]
#[derive(Debug, AstNode)]
pub struct While {
	pub cond: Node,
	pub body: Node,
}

#[has_attrs]
#[derive(Debug, AstNode)]
pub struct Break {}

#[has_attrs]
#[derive(Debug, AstNode)]
pub struct Continue {}

#[has_attrs]
#[derive(Debug, AstNode)]
pub struct Return {
	pub value: Option<Node>,
}
