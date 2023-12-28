use attr::{Attr, Attrs};
use std::fmt::Debug;
use sysyc_derive::{has_attrs, AstNode};
use utils::errors::Result;
use value::{BType, BinaryOp, FuncRetType, UnaryOp};

use crate::visitor::Visitor;

pub trait AstNode: Debug + Attrs {
	fn accept(&mut self, visitor: &mut dyn Visitor) -> Result<()>;
	fn is_end(&self) -> bool {
		false
	}
	fn is_init_val_list(&self) -> bool {
		false
	}
	fn mark_init_list_depth(&mut self) -> usize {
		0
	}
}

pub type Node = Box<dyn AstNode>;
pub type NodeList = Vec<Node>;

#[derive(Debug, AstNode)]
#[has_attrs]
pub struct Program {
	pub global_vars: NodeList,
	pub functions: NodeList,
	pub next_temp: u32,
}

#[derive(Debug, AstNode)]
#[has_attrs]
pub struct VarDef {
	pub ident: String,
	pub dim_list: NodeList,
	pub init: Option<Node>,
}

#[derive(Debug, AstNode)]
#[has_attrs]
pub struct VarDecl {
	pub is_const: bool,
	pub type_t: BType,
	pub defs: NodeList,
}

#[derive(Debug)]
#[has_attrs]
pub struct InitValList {
	pub val_list: NodeList,
}

impl AstNode for InitValList {
	fn accept(&mut self, visitor: &mut dyn Visitor) -> Result<()> {
		visitor.visit_init_val_list(self)
	}
	fn is_init_val_list(&self) -> bool {
		true
	}
	fn mark_init_list_depth(&mut self) -> usize {
		let mut max_depth = 0;
		for item in &mut self.val_list {
			let child_depth = item.mark_init_list_depth();
			max_depth = if max_depth < child_depth {
				child_depth
			} else {
				max_depth
			};
		}
		self.set_attr("initvallistdepth", Attr::InitValListDepth(max_depth + 1));
		max_depth + 1
	}
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
pub struct Variable {
	pub ident: String,
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
	pub ret_type: FuncRetType,
	pub ident: String,
	pub formal_params: NodeList,
	pub block: Node,
}

#[derive(Debug, AstNode)]
#[has_attrs]
pub struct FormalParam {
	pub type_t: BType,
	pub ident: String,
	pub dim_list: NodeList,
}

#[derive(Debug, AstNode)]
#[has_attrs]
pub struct Block {
	pub stmts: NodeList,
}

#[derive(Debug, AstNode)]
#[has_attrs]
pub struct If {
	pub cond: Node,
	pub body: Node,
	pub then: Option<Node>,
}

#[derive(Debug, AstNode)]
#[has_attrs]
pub struct While {
	pub cond: Node,
	pub body: Node,
}

#[derive(Debug, AstNode)]
#[has_attrs]
#[derive(Default)]
pub struct Break {}

#[derive(Debug, AstNode)]
#[has_attrs]
#[derive(Default)]
pub struct Continue {}

#[derive(Debug, AstNode)]
#[has_attrs]
pub struct Return {
	pub value: Option<Node>,
}
