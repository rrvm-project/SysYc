use scope::Scope;

use crate::{tree::*, visitor::Visitor};

impl AstNode for Program {
	fn accept(&mut self, visitor: &dyn Visitor, ctx: &mut dyn Scope) {
		visitor.visit_program(self, ctx)
	}
}

impl AstNode for VarDecl {
	fn accept(&mut self, visitor: &dyn Visitor, ctx: &mut dyn Scope) {
		visitor.visit_var_decl(self, ctx)
	}
}

impl AstNode for FuncDecl {
	fn accept(&mut self, visitor: &dyn Visitor, ctx: &mut dyn Scope) {
		visitor.visit_func_decl(self, ctx)
	}
}

impl AstNode for VarDef {
	fn accept(&mut self, visitor: &dyn Visitor, ctx: &mut dyn Scope) {
		visitor.visit_var_def(self, ctx)
	}
}

impl AstNode for InitValList {
	fn accept(&mut self, visitor: &dyn Visitor, ctx: &mut dyn Scope) {
		visitor.visit_init_val_list(self, ctx)
	}
}
