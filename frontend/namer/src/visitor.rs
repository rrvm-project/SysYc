#![allow(unused)]

use ast::{tree::*, Visitor};
use rrvm_symbol::{manager::SymbolManager, Symbol};
use scope::stack::ScopeStack;
use utils::errors::Result;
pub struct Namer {
	mgr: SymbolManager,
	ctx: ScopeStack,
}

impl Default for Namer {
	fn default() -> Self {
		Self::new()
	}
}

impl Namer {
	pub fn new() -> Self {
		Self {
			mgr: SymbolManager::new(),
			ctx: ScopeStack::new(),
		}
	}
	pub fn transform(&mut self, program: &mut Program) -> Result<()> {
		program.accept(self)
	}
}

impl Visitor for Namer {
	fn visit_program(&mut self, program: &mut Program) -> Result<()> {
		todo!()
	}
	fn visit_var_def(&mut self, val_decl: &mut VarDef) -> Result<()> {
		todo!()
	}
	fn visit_var_decl(&mut self, val_decl: &mut VarDecl) -> Result<()> {
		todo!()
	}
	fn visit_func_decl(&mut self, val_decl: &mut FuncDecl) -> Result<()> {
		todo!()
	}
	fn visit_init_val_list(&mut self, val_decl: &mut InitValList) -> Result<()> {
		todo!()
	}
	fn visit_literal_int(&mut self, val_decl: &mut LiteralInt) -> Result<()> {
		todo!()
	}
	fn visit_literal_float(&mut self, val_decl: &mut LiteralFloat) -> Result<()> {
		todo!()
	}
	fn visit_binary_expr(&mut self, val_decl: &mut BinaryExpr) -> Result<()> {
		todo!()
	}
	fn visit_unary_expr(&mut self, val_decl: &mut UnaryExpr) -> Result<()> {
		todo!()
	}
	fn visit_func_call(&mut self, val_decl: &mut FuncCall) -> Result<()> {
		todo!()
	}
	fn visit_formal_param(&mut self, val_decl: &mut FormalParam) -> Result<()> {
		todo!()
	}
	fn visit_variable(&mut self, val_decl: &mut Variable) -> Result<()> {
		todo!()
	}
	fn visit_block(&mut self, val_decl: &mut Block) -> Result<()> {
		todo!()
	}
	fn visit_if(&mut self, val_decl: &mut If) -> Result<()> {
		todo!()
	}
	fn visit_while(&mut self, val_decl: &mut While) -> Result<()> {
		todo!()
	}
	fn visit_continue(&mut self, val_decl: &mut Continue) -> Result<()> {
		todo!()
	}
	fn visit_break(&mut self, val_decl: &mut Break) -> Result<()> {
		todo!()
	}
	fn visit_return(&mut self, val_decl: &mut Return) -> Result<()> {
		todo!()
	}
}
