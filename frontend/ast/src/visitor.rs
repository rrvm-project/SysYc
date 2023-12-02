use crate::tree::*;
use utils::errors::Result;

#[rustfmt::skip]
pub trait Visitor {
	fn visit_program(&mut self, _node: &mut Program) -> Result<()> { unreachable!() }
	fn visit_var_def(&mut self, _node: &mut VarDef) -> Result<()> { unreachable!() }
	fn visit_var_decl(&mut self, _node: &mut VarDecl) -> Result<()> { unreachable!() }
	fn visit_func_decl(&mut self, _node: &mut FuncDecl) -> Result<()> { unreachable!() }
	fn visit_init_val_list(&mut self, _node: &mut InitValList) -> Result<()> { unreachable!() }
	fn visit_literal_int(&mut self, _node: &mut LiteralInt) -> Result<()> { unreachable!() }
	fn visit_literal_float(&mut self, _node: &mut LiteralFloat) -> Result<()> { unreachable!() }
	fn visit_binary_expr(&mut self, _node: &mut BinaryExpr) -> Result<()> { unreachable!() }
	fn visit_unary_expr(&mut self, _node: &mut UnaryExpr) -> Result<()> { unreachable!() }
	fn visit_func_call(&mut self, _node: &mut FuncCall) -> Result<()> { unreachable!() }
	fn visit_formal_param(&mut self, _node: &mut FormalParam) -> Result<()> { unreachable!() }
	fn visit_variable(&mut self, _node: &mut Variable) -> Result<()> { unreachable!() }
	fn visit_block(&mut self, _node: &mut Block) -> Result<()> { unreachable!() }
	fn visit_if(&mut self, _node: &mut If) -> Result<()> { unreachable!() }
	fn visit_while(&mut self, _node: &mut While) -> Result<()> { unreachable!() }
	fn visit_continue(&mut self, _node: &mut Continue) -> Result<()> { unreachable!() }
	fn visit_break(&mut self, _node: &mut Break) -> Result<()> { unreachable!() }
	fn visit_return(&mut self, _node: &mut Return) -> Result<()> { unreachable!() }
}
