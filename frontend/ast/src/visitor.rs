use crate::tree::*;
use utils::errors::Result;

pub trait Visitor {
	fn visit_program(&mut self, program: &mut Program) -> Result<()>;
	fn visit_var_def(&mut self, val_decl: &mut VarDef) -> Result<()>;
	fn visit_var_decl(&mut self, val_decl: &mut VarDecl) -> Result<()>;
	fn visit_func_decl(&mut self, val_decl: &mut FuncDecl) -> Result<()>;
	fn visit_init_val_list(&mut self, val_decl: &mut InitValList) -> Result<()>;
	fn visit_literal_int(&mut self, val_decl: &mut LiteralInt) -> Result<()>;
	fn visit_literal_float(&mut self, val_decl: &mut LiteralFloat) -> Result<()>;
	fn visit_binary_expr(&mut self, val_decl: &mut BinaryExpr) -> Result<()>;
	fn visit_unary_expr(&mut self, val_decl: &mut UnaryExpr) -> Result<()>;
	fn visit_func_call(&mut self, val_decl: &mut FuncCall) -> Result<()>;
	fn visit_formal_param(&mut self, val_decl: &mut FormalParam) -> Result<()>;
	fn visit_variable(&mut self, val_decl: &mut Variable) -> Result<()>;
	fn visit_block(&mut self, val_decl: &mut Block) -> Result<()>;
	fn visit_if(&mut self, val_decl: &mut If) -> Result<()>;
	fn visit_while(&mut self, val_decl: &mut While) -> Result<()>;
	fn visit_continue(&mut self, val_decl: &mut Continue) -> Result<()>;
	fn visit_break(&mut self, val_decl: &mut Break) -> Result<()>;
	fn visit_return(&mut self, val_decl: &mut Return) -> Result<()>;
}

/*

here is how to implement a visitor

pub struct PrintVisitor {

}

impl Visitor for PrintVisitor {
	fn visitProgram(&self, program: &mut Program, ctx: &mut dyn Context) {
		program.accept(self, ctx)
	}
}

*/
