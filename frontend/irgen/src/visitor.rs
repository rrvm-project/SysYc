#![allow(unused)]

use std::collections::HashMap;

use ast::{
	tree::{AstRetType::Empty, *},
	Visitor,
};
use attr::Attrs;
use rrvm::program::RrvmProgram;
use rrvm_symbol::{manager::SymbolManager, FuncSymbol, Symbol, VarSymbol};
use utils::{errors::Result, Label, SysycError::TypeError};
use value::{
	calc_type::{to_rval, type_binaryop},
	BType, BinaryOp, FuncType, Value, VarType,
};

pub struct IRGenerator {
	loop_label_stack: Vec<(Label, Label)>,
	program: RrvmProgram,
}

impl Default for IRGenerator {
	fn default() -> Self {
		Self::new()
	}
}

impl IRGenerator {
	pub fn new() -> Self {
		Self {
			program: RrvmProgram::new(),
			loop_label_stack: Vec::new(),
		}
	}
	pub fn transform(&mut self, program: &mut Program) -> Result<AstRetType> {
		program.accept(self)
	}
}

impl Visitor for IRGenerator {
	fn visit_program(&mut self, node: &mut Program) -> Result<AstRetType> {
		for v in node.comp_units.iter_mut() {
			v.accept(self)?;
		}
		Ok(Empty)
	}
	fn visit_func_decl(&mut self, node: &mut FuncDecl) -> Result<AstRetType> {
		node.block.accept(self)
	}
	fn visit_var_def(&mut self, node: &mut VarDef) -> Result<AstRetType> {
		node.init.as_mut().map(|v| v.accept(self));
		Ok(Empty)
	}
	fn visit_var_decl(&mut self, node: &mut VarDecl) -> Result<AstRetType> {
		for var_def in node.defs.iter_mut() {
			var_def.accept(self)?;
		}
		Ok(Empty)
	}
	fn visit_init_val_list(
		&mut self,
		node: &mut InitValList,
	) -> Result<AstRetType> {
		for val in node.val_list.iter_mut() {
			val.accept(self)?;
		}
		Ok(Empty)
	}
	fn visit_literal_int(&mut self, node: &mut LiteralInt) -> Result<AstRetType> {
		Ok(Empty)
	}
	fn visit_literal_float(
		&mut self,
		node: &mut LiteralFloat,
	) -> Result<AstRetType> {
		Ok(Empty)
	}
	fn visit_binary_expr(&mut self, node: &mut BinaryExpr) -> Result<AstRetType> {
		node.lhs.accept(self);
		node.rhs.accept(self);
		Ok(Empty)
	}
	fn visit_unary_expr(&mut self, node: &mut UnaryExpr) -> Result<AstRetType> {
		node.rhs.accept(self);
		Ok(Empty)
	}
	fn visit_func_call(&mut self, node: &mut FuncCall) -> Result<AstRetType> {
		Ok(Empty)
	}
	fn visit_formal_param(
		&mut self,
		node: &mut FormalParam,
	) -> Result<AstRetType> {
		unreachable!()
	}
	fn visit_variable(&mut self, node: &mut Variable) -> Result<AstRetType> {
		Ok(Empty)
	}
	fn visit_block(&mut self, node: &mut Block) -> Result<AstRetType> {
		for stmt in node.stmts.iter_mut() {
			stmt.accept(self)?;
		}
		Ok(Empty)
	}
	fn visit_if(&mut self, node: &mut If) -> Result<AstRetType> {
		node.cond.accept(self)?;
		node.body.accept(self)?;
		if let Some(then) = &mut node.then {
			then.accept(self)?;
		}
		Ok(Empty)
	}
	fn visit_while(&mut self, node: &mut While) -> Result<AstRetType> {
		node.cond.accept(self)?;
		node.body.accept(self)
	}
	fn visit_continue(&mut self, node: &mut Continue) -> Result<AstRetType> {
		Ok(Empty)
	}
	fn visit_break(&mut self, node: &mut Break) -> Result<AstRetType> {
		Ok(Empty)
	}
	fn visit_return(&mut self, node: &mut Return) -> Result<AstRetType> {
		if let Some(val) = &mut node.value {
			val.accept(self)?;
		}
		Ok(Empty)
	}
}
