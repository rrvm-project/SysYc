use crate::tree::*;
use utils::SysycError;

// TODO：这里实现泛型很困难，考虑将ctx实现成全局变量
pub trait Visitor {

	fn visit_program(&self, program: &mut Program) -> Result<(), SysycError>;
	fn visit_var_def(&self, val_decl: &mut VarDef) -> Result<(), SysycError>;
	fn visit_var_decl(&self, val_decl: &mut VarDecl) -> Result<(), SysycError>;
	fn visit_func_decl(&self, val_decl: &mut FuncDecl) -> Result<(), SysycError>;
	fn visit_init_val_list(&self,	val_decl: &mut InitValList) -> Result<(), SysycError>;
	fn visit_literal_int(&self,	val_decl: &mut LiteralInt) -> Result<(), SysycError>;
	fn visit_literal_float(&self,	val_decl: &mut LiteralFloat) -> Result<(), SysycError>;
	fn visit_binary_expr(&self,	val_decl: &mut BinaryExpr) -> Result<(), SysycError>;
	fn visit_unary_expr(&self, val_decl: &mut UnaryExpr) -> Result<(), SysycError>;
	fn visit_func_call(&self,	val_decl: &mut FuncCall) -> Result<(), SysycError>;
	fn visit_formal_param(&self,	val_decl: &mut FormalParam) -> Result<(), SysycError>;
	fn visit_lval(&self, val_decl: &mut Lval) -> Result<(), SysycError>;
	fn visit_block(&self,	val_decl: &mut Block) -> Result<(), SysycError>;
	fn visit_if(&self, val_decl: &mut If) -> Result<(), SysycError>;
	fn visit_while(&self,	val_decl: &mut While) -> Result<(), SysycError>;
	fn visit_continue(&self, val_decl: &mut Continue) -> Result<(), SysycError>;
	fn visit_break(&self,	val_decl: &mut Break) -> Result<(), SysycError>;
	fn visit_return(&self, val_decl: &mut Return) -> Result<(), SysycError>;
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

