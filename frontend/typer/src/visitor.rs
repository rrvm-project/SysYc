#![allow(unused)]

use std::collections::HashMap;

use ast::{tree::*, Visitor};
use attr::Attrs;
use rrvm_symbol::{manager::SymbolManager, FuncSymbol, Symbol, VarSymbol};
use scope::{scope::Scope, stack::ScopeStack};
use utils::{errors::Result, SysycError::TypeError};
use value::{
	calc_type::{to_rval, type_binaryop},
	BType, BinaryOp, FuncType, Value, VarType,
};

pub struct Typer {}

impl Default for Typer {
	fn default() -> Self {
		Self::new()
	}
}

impl Typer {
	pub fn new() -> Self {
		Self {}
	}
	pub fn transform(&mut self, program: &mut Program) -> Result<()> {
		program.accept(self)
	}
}

impl Visitor for Typer {
	fn visit_program(&mut self, node: &mut Program) -> Result<()> {
		for v in node.comp_units.iter_mut() {
			v.accept(self)?
		}
		Ok(())
	}
	fn visit_func_decl(&mut self, node: &mut FuncDecl) -> Result<()> {
		node.block.accept(self)
	}
	fn visit_var_def(&mut self, node: &mut VarDef) -> Result<()> {
		node.init.as_mut().map(|v| v.accept(self));
		Ok(())
	}
	fn visit_var_decl(&mut self, node: &mut VarDecl) -> Result<()> {
		for var_def in node.defs.iter_mut() {
			var_def.accept(self)?;
		}
		Ok(())
	}
	fn visit_init_val_list(&mut self, node: &mut InitValList) -> Result<()> {
		for val in node.val_list.iter_mut() {
			val.accept(self)?;
		}
		Ok(())
	}
	fn visit_literal_int(&mut self, node: &mut LiteralInt) -> Result<()> {
		node.set_attr("type", VarType::new_int().into());
		Ok(())
	}
	fn visit_literal_float(&mut self, node: &mut LiteralFloat) -> Result<()> {
		node.set_attr("type", VarType::new_float().into());
		Ok(())
	}
	fn visit_binary_expr(&mut self, node: &mut BinaryExpr) -> Result<()> {
		node.lhs.accept(self);
		node.rhs.accept(self);
		let lhs = node.lhs.get_attr("type").ok_or(TypeError(
			" void value not ignored as it ought to be".to_string(),
		))?;
		let rhs = node.rhs.get_attr("type").ok_or(TypeError(
			" void value not ignored as it ought to be".to_string(),
		))?;
		let type_t = type_binaryop(&lhs.into(), node.op, &rhs.into())?;
		node.set_attr("type", type_t.into());
		Ok(())
	}
	fn visit_unary_expr(&mut self, node: &mut UnaryExpr) -> Result<()> {
		node.rhs.accept(self);
		let rhs = node.rhs.get_attr("type").ok_or(TypeError(
			" void value not ignored as it ought to be".to_string(),
		))?;
		node.set_attr("type", to_rval(&rhs.into()).into());
		Ok(())
	}
	fn visit_func_call(&mut self, node: &mut FuncCall) -> Result<()> {
		let symbol: FuncSymbol = node.get_attr("func_symbol").unwrap().into();
		let (_, params) = symbol.var_type;
		if node.params.len() != params.len() {
			return Err(TypeError(format!(
				"unmatch numbers of params for function {}",
				node.ident
			)));
		}

		for (x_t, y) in params.iter().zip(node.params.iter()) {
			let y_t: VarType = y.get_attr("type").unwrap().into();
			let err_msg =
				format!("expected `{}` but argument is of type `{}`", x_t, y_t);
			if x_t.dims.len() != y_t.dims.iter().len()
				|| x_t.dims.iter().skip(1).ne(y_t.dims.iter().skip(1))
			{
				return Err(TypeError(err_msg));
			}
		}
		Ok(())
	}
	fn visit_formal_param(&mut self, node: &mut FormalParam) -> Result<()> {
		unreachable!()
	}
	fn visit_variable(&mut self, node: &mut Variable) -> Result<()> {
		Ok(())
	}
	fn visit_block(&mut self, node: &mut Block) -> Result<()> {
		for stmt in node.stmts.iter_mut() {
			stmt.accept(self)?;
		}
		Ok(())
	}
	fn visit_if(&mut self, node: &mut If) -> Result<()> {
		node.cond.accept(self)?;
		node.body.accept(self)?;
		if let Some(then) = &mut node.then {
			then.accept(self)?;
		}
		Ok(())
	}
	fn visit_while(&mut self, node: &mut While) -> Result<()> {
		node.cond.accept(self)?;
		node.body.accept(self)
	}
	fn visit_continue(&mut self, node: &mut Continue) -> Result<()> {
		Ok(())
	}
	fn visit_break(&mut self, node: &mut Break) -> Result<()> {
		Ok(())
	}
	fn visit_return(&mut self, node: &mut Return) -> Result<()> {
		if let Some(val) = &mut node.value {
			val.accept(self)?;
		}
		Ok(())
	}
}
