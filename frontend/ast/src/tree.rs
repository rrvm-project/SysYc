use scope::Scope;
use std::fmt::Debug;
use sysyc_derive::{has_attrs, AstNode};
use utils::{Attr, Attrs};

use crate::{visitor::Visitor, BinaryOp, Type, UnaryOp};

pub trait AstNode: Debug + Attrs {
	fn accept(&mut self, visitor: &dyn Visitor, ctx: &mut dyn Scope);
}

pub type Node = Box<dyn AstNode>;
pub type NodeList = Vec<Node>;

#[derive(Debug, AstNode)]
#[has_attrs]
pub struct Program {
	pub comp_units: NodeList,
}

#[derive(Debug, AstNode)]
#[has_attrs]
pub struct DimList {
	pub exprs: NodeList,
}

#[derive(Debug, AstNode)]
#[has_attrs]
pub struct VarDef {
	pub ident: String,
	pub dim_list: Option<Node>,
	pub init: Option<Node>,
}

#[derive(Debug, AstNode)]
#[has_attrs]
pub struct VarDecl {
	pub is_const: bool,
	pub type_t: Type,
	pub defs: NodeList,
}

#[derive(Debug, AstNode)]
#[has_attrs]
pub struct InitValList {
	pub val_list: NodeList,
}

#[derive(Debug, AstNode)]
#[has_attrs]
pub struct LiteralInt {
	pub value: i32,
}

#[derive(Debug, AstNode)]
#[has_attrs]
pub struct LiteralFloat {
	pub value: f32,
}

#[derive(Debug, AstNode)]
#[has_attrs]
pub struct BinaryExpr {
	pub lhs: Node,
	pub op: BinaryOp,
	pub rhs: Node,
}

#[derive(Debug, AstNode)]
#[has_attrs]
pub struct UnaryExpr {
	pub op: UnaryOp,
	pub rhs: Node,
}

#[derive(Debug, AstNode)]
#[has_attrs]
pub struct FuncCall {
	pub ident: String,
	pub params: NodeList,
}

#[derive(Debug, AstNode)]
#[has_attrs]
pub struct FuncDecl {
	pub ident: String,
	pub formal_params: NodeList,
}

#[derive(Debug, AstNode)]
#[has_attrs]
pub struct FormalParam {
	pub type_t: Type,
	pub ident: String,
	pub dim_list: Option<Node>,
}

#[derive(Debug, AstNode)]
#[has_attrs]
pub struct Lval {
	pub ident: String,
	pub dim_list: Option<Node>,
}
