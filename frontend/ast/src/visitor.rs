use scope::Scope;

use crate::tree::*;

pub trait Visitor {
	fn visit_program(&self, program: &mut Program, ctx: &mut dyn Scope);
	fn visit_var_def(&self, val_decl: &mut VarDef, ctx: &mut dyn Scope);
	fn visit_var_decl(&self, val_decl: &mut VarDecl, ctx: &mut dyn Scope);
	fn visit_func_decl(&self, val_decl: &mut FuncDecl, ctx: &mut dyn Scope);
	fn visit_init_val_list(&self,	val_decl: &mut InitValList, ctx: &mut dyn Scope);
	fn visit_literal_int(&self,	val_decl: &mut LiteralInt, ctx: &mut dyn Scope);
	fn visit_literal_float(&self,	val_decl: &mut LiteralFloat, ctx: &mut dyn Scope);
	fn visit_binary_expr(&self,	val_decl: &mut BinaryExpr, ctx: &mut dyn Scope);
	fn visit_unary_expr(&self, val_decl: &mut UnaryExpr, ctx: &mut dyn Scope);
	fn visit_func_call(&self,	val_decl: &mut FuncCall, ctx: &mut dyn Scope);
	fn visit_formal_param(&self,	val_decl: &mut FormalParam, ctx: &mut dyn Scope);
	fn visit_lval(&self, val_decl: &mut Lval, ctx: &mut dyn Scope);
	fn visit_block(&self,	val_decl: &mut Block, ctx: &mut dyn Scope);
	fn visit_if(&self, val_decl: &mut If, ctx: &mut dyn Scope);
	fn visit_while(&self,	val_decl: &mut While, ctx: &mut dyn Scope);
	fn visit_continue(&self, val_decl: &mut Continue, ctx: &mut dyn Scope);
	fn visit_break(&self,	val_decl: &mut Break, ctx: &mut dyn Scope);
	fn visit_return(&self, val_decl: &mut Return, ctx: &mut dyn Scope);
}


/*

here is how to implement a visitor

pub struct PrintVisitor {

}

impl Visitor for PrintVisitor {
  fn visitProgram(&self, program: &mut Program, ctx: &mut dyn Scope) {
    program.accept(self, ctx)
  }
}

*/

