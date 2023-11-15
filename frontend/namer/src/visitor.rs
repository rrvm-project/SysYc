#![allow(unused)]

use std::collections::HashMap;

use ast::{tree::*, Visitor};
use rrvm_symbol::{manager::SymbolManager, Symbol};
use scope::{scope::Scope, stack::ScopeStack};
use utils::errors::Result;
use value::Value;
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
	fn visit_program(&mut self, node: &mut Program) -> Result<()> {
		self.ctx.push();
		for v in node.comp_units.iter_mut() {
			v.accept(self)?
		}
		Ok(())
	}
	fn visit_func_decl(&mut self, node: &mut FuncDecl) -> Result<()> {
		for param in node.formal_params.iter_mut() {
			param.accept(self)?
		}
		// let v = HashMap::new();
		// let val= Value::FloatPtr(Vec::new(), v);
		// let func_type = node.formal_params.iter().map(|v| v.get_attr(name));

		// node.formal_params.iter_mut().for_each(|v| v.accept(self));
		// node.func_type;
		// node.formal_params.
		Ok(())
		// todo!()
	}
	fn visit_var_def(&mut self, node: &mut VarDef) -> Result<()> {
		todo!()
	}
	fn visit_var_decl(&mut self, node: &mut VarDecl) -> Result<()> {
		todo!()
	}
	fn visit_init_val_list(&mut self, node: &mut InitValList) -> Result<()> {
		todo!()
	}
	fn visit_literal_int(&mut self, node: &mut LiteralInt) -> Result<()> {
		todo!()
	}
	fn visit_literal_float(&mut self, node: &mut LiteralFloat) -> Result<()> {
		todo!()
	}
	fn visit_binary_expr(&mut self, node: &mut BinaryExpr) -> Result<()> {
		todo!()
	}
	fn visit_unary_expr(&mut self, node: &mut UnaryExpr) -> Result<()> {
		todo!()
	}
	fn visit_func_call(&mut self, node: &mut FuncCall) -> Result<()> {
		todo!()
	}
	fn visit_formal_param(&mut self, node: &mut FormalParam) -> Result<()> {
		todo!()
	}
	fn visit_variable(&mut self, node: &mut Variable) -> Result<()> {
		todo!()
	}
	fn visit_block(&mut self, node: &mut Block) -> Result<()> {
		todo!()
	}
	fn visit_if(&mut self, node: &mut If) -> Result<()> {
		todo!()
	}
	fn visit_while(&mut self, node: &mut While) -> Result<()> {
		todo!()
	}
	fn visit_continue(&mut self, node: &mut Continue) -> Result<()> {
		todo!()
	}
	fn visit_break(&mut self, node: &mut Break) -> Result<()> {
		todo!()
	}
	fn visit_return(&mut self, node: &mut Return) -> Result<()> {
		todo!()
	}
}
