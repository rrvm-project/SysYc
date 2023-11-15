use crate::tree::*;
use utils::errors::Result;

pub trait Visitor {
	fn visit_program(&mut self, node: &mut Program) -> Result<()>;
	fn visit_var_def(&mut self, node: &mut VarDef) -> Result<()>;
	fn visit_var_decl(&mut self, node: &mut VarDecl) -> Result<()>;
	fn visit_func_decl(&mut self, node: &mut FuncDecl) -> Result<()>;
	fn visit_init_val_list(&mut self, node: &mut InitValList) -> Result<()>;
	fn visit_literal_int(&mut self, node: &mut LiteralInt) -> Result<()>;
	fn visit_literal_float(&mut self, node: &mut LiteralFloat) -> Result<()>;
	fn visit_binary_expr(&mut self, node: &mut BinaryExpr) -> Result<()>;
	fn visit_unary_expr(&mut self, node: &mut UnaryExpr) -> Result<()>;
	fn visit_func_call(&mut self, node: &mut FuncCall) -> Result<()>;
	fn visit_formal_param(&mut self, node: &mut FormalParam) -> Result<()>;
	fn visit_variable(&mut self, node: &mut Variable) -> Result<()>;
	fn visit_block(&mut self, node: &mut Block) -> Result<()>;
	fn visit_if(&mut self, node: &mut If) -> Result<()>;
	fn visit_while(&mut self, node: &mut While) -> Result<()>;
	fn visit_continue(&mut self, node: &mut Continue) -> Result<()>;
	fn visit_break(&mut self, node: &mut Break) -> Result<()>;
	fn visit_return(&mut self, node: &mut Return) -> Result<()>;
}
