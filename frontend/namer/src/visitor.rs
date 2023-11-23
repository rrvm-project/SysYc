#![allow(unused)]

use std::{collections::HashMap, process::exit};

use ast::{tree::*, Visitor};
use attr::{Attr, Attrs};
use rrvm_symbol::{manager::SymbolManager, FuncSymbol, Symbol, VarSymbol};
use scope::{scope::Scope, stack::ScopeStack};
use utils::{errors::Result, init_value_item, SysycError::TypeError};
use value::{
	calc::{exec_binaryop, exec_unaryop},
	typer::{type_for_binary, type_for_unary},
	BType, BinaryOp, FuncType, Value, VarType,
};

use crate::utils_namer::*;

#[derive(Debug)]
struct InitListContext {
	pub dims_alignment: Vec<usize>,
	pub used_space: usize,
	pub init_values: HashMap<usize, Value>,
	pub expect_const: bool,
	pub target_type: BType,
	pub total_size: usize,
}

pub struct Namer {
	mgr: SymbolManager,
	ctx: ScopeStack,
	cur_type: Option<(bool, BType)>,
	init_list_context: Option<InitListContext>,
	init_value_list: HashMap<i32, HashMap<usize, Value>>,
	global_value_list: Vec<(String, Vec<init_value_item::InitValueItem>)>,
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
			cur_type: None,
			init_list_context: None,
			init_value_list: HashMap::new(),
			global_value_list: vec![],
		}
	}
	pub fn transform(&mut self, program: &mut Program) -> Result<()> {
		program.accept(self)
	}
}

impl Namer {
	fn visit_dim_list(&mut self, node_list: &mut NodeList) -> Result<Vec<usize>> {
		let mut dim_list: Vec<usize> = Vec::new();
		for dim in node_list.iter_mut() {
			dim.accept(self)?;
			let value: Value = dim
				.get_attr("value")
				.ok_or(TypeError(
					"The length of array must be constant integer".to_string(),
				))?
				.into();
			dim_list.push(value.to_int()? as usize);
		}
		Ok(dim_list)
	}
}

impl Visitor for Namer {
	fn visit_program(&mut self, node: &mut Program) -> Result<()> {
		self.ctx.push();
		for v in node.comp_units.iter_mut() {
			v.accept(self)?
		}

		// println!("init values at end{:?}", self.init_value_list);

		for (name, symbol) in self.ctx.report_all_global() {
			if !symbol.is_global {
				continue;
			}
			let mut size = 1;

			for item in &symbol.var_type.2 {
				size *= *item;
			}
			let value_map = self.init_value_list.get(&symbol.id).unwrap();
			self
				.global_value_list
				.push((name.clone(), get_global_init_value(value_map, size)))
		}

		// println!("global_var{:?}", self.global_value_list);

		self.ctx.pop();

		Ok(())
	}
	fn visit_func_decl(&mut self, node: &mut FuncDecl) -> Result<()> {
		let mut func_type = Vec::new();
		for param in node.formal_params.iter_mut() {
			param.accept(self)?;
			func_type.push(param.get_attr("type").unwrap().into());
		}
		let func_type: FuncType = (node.ret_type, func_type);
		let symbol = self.mgr.new_symbol(Some(node.ident.clone()), func_type);
		self.ctx.set_func(&node.ident, symbol)?;
		self.ctx.push();
		node.block.accept(self)?;
		self.ctx.pop()
	}
	fn visit_var_def(&mut self, node: &mut VarDef) -> Result<()> {
		let dim_list = self.visit_dim_list(&mut node.dim_list)?;
		let is_array = !dim_list.is_empty();

		let mut alignment = vec![];
		alignment.push(1);

		for i in 1..(dim_list.len()) {
			let current_size = dim_list[dim_list.len() - i];
			alignment.push(alignment[i - 1] * current_size);
		}

		let total_size = if is_array {
			alignment[alignment.len() - 1] * dim_list[0]
		} else {
			1
		};

		let (is_const, btype) = self.cur_type.unwrap();
		let var_type = (is_const, btype, dim_list);
		let var_type_size = var_type.2.iter().product::<usize>();
		let mut symbol =
			self.mgr.new_symbol(Some(node.ident.clone()), var_type.clone());
		symbol.is_global = self.ctx.is_global();

		self.init_list_context = InitListContext {
			dims_alignment: alignment,
			used_space: 0,
			init_values: HashMap::new(),
			expect_const: is_const || self.ctx.is_global(),
			target_type: btype,
			total_size,
		}
		.into();

		self.ctx.set_val(&node.ident, symbol.clone());
		node.set_attr("type", var_type.into());
		let symbol_id = symbol.id;
		node.set_attr("symbol", symbol.into());

		if let Some(init_value) = &mut node.init {
			init_value.accept(self)?;
			if is_array {
				init_value.accept(self)?;
			} else if let Some(attr::Attr::Value(value)) =
				init_value.get_attr("value")
			{
				self
					.init_list_context
					.as_mut()
					.unwrap()
					.init_values
					.insert(0, value.to_target_btype(btype)?);
			}
		}

		if self.ctx.is_global() || is_const {
			// Add by cyh
			if self.ctx.is_global() {
				node.set_attr(
					"global_value",
					Attr::GlobalValue(get_global_init_value(
						&self.init_list_context.as_mut().unwrap().init_values,
						var_type_size,
					)),
				)
			}
			// Reformed by cyh
			self.init_value_list.insert(
				symbol_id,
				self.init_list_context.take().unwrap().init_values,
			);
		}

		// println!("init values{:?}", self.init_value_list);
		// self.init_list_context = None;

		Ok(())
	}
	fn visit_var_decl(&mut self, node: &mut VarDecl) -> Result<()> {
		self.cur_type = Some((node.is_const, node.type_t));
		for var_def in node.defs.iter_mut() {
			var_def.accept(self)?;
		}
		self.cur_type = None;
		Ok(())
	}

	fn visit_init_val_list(&mut self, node: &mut InitValList) -> Result<()> {
		if self.init_list_context.is_none() {
			unreachable!("init list must in a var_decl of array");
		};

		let first_pass = node.get_attr("init_list_height").is_none();

		if first_pass {
			let mut max_depth_of_child: usize = 0;
			for item in &mut node.val_list {
				item.accept(self)?;
				if let Some(attr::Attr::InitListHeight(height)) =
					item.get_attr("init_list_height")
				{
					if max_depth_of_child < *height {
						max_depth_of_child = *height;
					}
				} else {
					item.set_attr("init_list_height", attr::Attr::InitListHeight(0));
				}
			}
			node.set_attr(
				"init_list_height",
				attr::Attr::InitListHeight(max_depth_of_child + 1),
			);
			Ok(())
		} else {
			for item in &mut node.val_list {
				if let Some(attr::Attr::InitListHeight(height)) =
					item.get_attr("init_list_height")
				{
					let alignment =
						self.init_list_context.as_ref().unwrap().dims_alignment[*height];
					let total_size = self.init_list_context.as_ref().unwrap().total_size;
					let used = self.init_list_context.as_ref().unwrap().used_space;
					let blank = used % alignment;
					let position = used + if blank == 0 { 0 } else { alignment - blank };
					let height = *height;
					if height == 0 {
						self.init_list_context.as_mut().unwrap().used_space =
							position + alignment;

						let target_type =
							self.init_list_context.as_ref().unwrap().target_type;

						if let Some(attr::Attr::Value(v)) = item.get_attr("value") {
							self
								.init_list_context
								.as_mut()
								.unwrap()
								.init_values
								.insert(position, v.to_target_btype(target_type)?);
						} else if self.init_list_context.as_ref().unwrap().expect_const {
							dbg!(&item);
							return Err(utils::SysycError::SyntaxError(
								"failed to get const value of above for array init".to_string(),
							));
						}
					} else {
						self.init_list_context.as_mut().unwrap().used_space = position;
					}
					item.set_attr(
						"init_value_index",
						attr::Attr::InitListPosition(position),
					);

					if (position >= total_size) {
						return Err(utils::SysycError::SyntaxError(
							"too many init value".to_string(),
						));
					}

					item.accept(self)?;

					let used = self.init_list_context.as_ref().unwrap().used_space;
					let blank = used % alignment;
					let position = used + if blank == 0 { 0 } else { alignment - blank };
					if height != 0 {
						self.init_list_context.as_mut().unwrap().used_space = position;
					}
				} else {
					unreachable!("should be added during first pass")
				}
			}

			Ok(())
		}
	}
	fn visit_literal_int(&mut self, node: &mut LiteralInt) -> Result<()> {
		let value: Value = node.value.into();
		node.set_attr("value", value.into());
		node.set_attr("type", (false, BType::Int, vec![]).into());
		Ok(())
	}
	fn visit_literal_float(&mut self, node: &mut LiteralFloat) -> Result<()> {
		let value: Value = node.value.into();
		node.set_attr("value", value.into());
		node.set_attr("type", (false, BType::Float, vec![]).into());
		Ok(())
	}
	fn visit_binary_expr(&mut self, node: &mut BinaryExpr) -> Result<()> {
		node.lhs.accept(self)?;
		node.rhs.accept(self)?;
		if node.op != BinaryOp::Assign {
			let lhs = node.lhs.get_attr("value");
			let rhs = node.rhs.get_attr("value");
			if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
				let value = exec_binaryop(&lhs.into(), node.op, &rhs.into())?;
				node.set_attr("value", value.into());
			};

			let lhs = node.lhs.get_attr("type");
			let rhs = node.rhs.get_attr("type");
			if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
				let value = type_for_binary(&lhs.into(), node.op, &rhs.into())?;
				node.set_attr("type", value.into());
			};
		} else {
			if let Some(rhs) = node.rhs.get_attr("type") {
				node.set_attr("type", rhs.clone());
			};
			if let Some(rhs) = node.rhs.get_attr("value") {
				node.set_attr("value", rhs.clone());
			};
		}

		Ok(())
	}
	fn visit_unary_expr(&mut self, node: &mut UnaryExpr) -> Result<()> {
		node.rhs.accept(self)?;
		if let Some(rhs) = node.rhs.get_attr("value") {
			let value = exec_unaryop(node.op, &rhs.into())?;
			node.set_attr("value", value.into());
		}

		if let Some(rhs) = node.rhs.get_attr("type") {
			let typer = type_for_unary(&rhs.into(), node.op)?;
			node.set_attr("type", typer.into());
		}
		Ok(())
	}
	fn visit_func_call(&mut self, node: &mut FuncCall) -> Result<()> {
		let symbol = self.ctx.find_func(&node.ident)?.clone();
		match &symbol.var_type.0 {
			value::FuncRetType::Float => {
				node.set_attr("type", (false, value::BType::Float, vec![]).into())
			}
			value::FuncRetType::Int => {
				node.set_attr("type", (false, value::BType::Int, vec![]).into())
			}
			_ => {}
		}
		node.set_attr("func_symbol", symbol.into());

		for param in node.params.iter_mut() {
			param.accept(self)?;
		}
		Ok(())
	}
	fn visit_formal_param(&mut self, node: &mut FormalParam) -> Result<()> {
		let dim_list = self.visit_dim_list(&mut node.dim_list)?;
		let var_type = (false, node.type_t, dim_list);
		let symbol = self.mgr.new_symbol(None, var_type.clone());
		self.ctx.set_val(&node.ident, symbol.clone());
		node.set_attr("symbol", symbol.into());
		node.set_attr("type", var_type.into());
		// if node.dim_list.
		Ok(())
	}
	fn visit_variable(&mut self, node: &mut Variable) -> Result<()> {
		// TODO : 设定合适的type和value
		let symbol = self.ctx.find_val(&node.ident)?.clone();

		node.set_attr("type", attr::Attr::VarType(symbol.var_type.clone()));

		if symbol.var_type.0 {
			//const
			if let Some(value) = self.init_value_list.get(&symbol.id) {
				// scalar
				if symbol.var_type.2.is_empty() {
					if let Some(inner_value) = value.get(&0) {
						node.set_attr("value", attr::Attr::Value(inner_value.clone()))
					} else {
						//TODO 是否什么都不做?
					}
				} else {
					// array
					node.set_attr(
						"value",
						attr::Attr::Value(get_value_for_calc(
							symbol.var_type.1,
							&symbol.var_type.2,
							value,
						)?),
					);
				}
			}
		}
		node.set_attr("symbol", symbol.into());
		Ok(())
	}
	fn visit_block(&mut self, node: &mut Block) -> Result<()> {
		self.ctx.push();
		for stmt in node.stmts.iter_mut() {
			stmt.accept(self)?;
		}
		self.ctx.pop()?;
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
