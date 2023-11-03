use scope::Scope;
use std::fmt::Debug;
use sysyc_derive::has_attrs;
use utils::{Attr, Attrs};

use crate::{visitor::Visitor, Type};

pub trait AstNode: Debug + Attrs {
	fn accept(&mut self, visitor: &dyn Visitor, ctx: &mut dyn Scope);
}

pub type Node = Box<dyn AstNode>;
pub type NodeList = Vec<Node>;

#[derive(Debug)]
#[has_attrs]
pub struct Program {
	pub comp_units: NodeList,
}

#[derive(Debug)]
#[has_attrs]
pub struct DimList {
  pub exprs: NodeList,
}

#[derive(Debug)]
#[has_attrs]
pub struct VarDef {
	pub ident: String,
  pub dim_list: Option<Node>,
	pub init: Option<Node>,
}

#[derive(Debug)]
#[has_attrs]
pub struct VarDecl {
	pub is_const: bool,
	pub type_t: Type,
	pub defs: NodeList,
}

#[derive(Debug)]
#[has_attrs]
pub struct InitValList {
  pub val_list: NodeList,
}

#[derive(Debug)]
#[has_attrs]
pub struct FuncDecl {}
