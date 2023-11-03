use scope::Scope;

use crate::tree::*;

pub trait Visitor {
	fn visit_program(&self, program: &mut Program, ctx: &mut dyn Scope);
	fn visit_var_def(&self, val_decl: &mut VarDef, ctx: &mut dyn Scope);
	fn visit_var_decl(&self, val_decl: &mut VarDecl, ctx: &mut dyn Scope);
	fn visit_func_decl(&self, val_decl: &mut FuncDecl, ctx: &mut dyn Scope);
	fn visit_init_val_list(&self,	val_decl: &mut InitValList, ctx: &mut dyn Scope);
	fn visit_dim_list(&self,	val_decl: &mut DimList, ctx: &mut dyn Scope);
	fn visit_literal_int(&self,	val_decl: &mut LiteralInt, ctx: &mut dyn Scope);
	fn visit_literal_float(&self,	val_decl: &mut LiteralFloat, ctx: &mut dyn Scope);
	fn visit_binary_expr(&self,	val_decl: &mut BinaryExpr, ctx: &mut dyn Scope);
	fn visit_unary_expr(&self,	val_decl: &mut UnaryExpr, ctx: &mut dyn Scope);
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

