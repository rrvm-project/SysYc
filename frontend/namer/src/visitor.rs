use std::collections::HashMap;

use ast::{shirink, tree::*, Visitor};
use attr::Attrs;
use rrvm_symbol::manager::SymbolManager;
use scope::stack::ScopeStack;
use utils::{
	errors::Result, init_value_item::InitValueItem, SysycError::TypeError,
};
use value::{
	calc::{exec_binaryop, exec_unaryop},
	BType, BinaryOp, FuncType, Value, VarType,
};

#[derive(Default)]
pub struct Namer {
	mgr: SymbolManager,
	ctx: ScopeStack,
	cur_type: Option<(bool, BType)>,
	dim_list_processing: Option<Vec<usize>>,
	alignment_processing: Option<Vec<usize>>,
	current_index_processing: usize,
	const_processing: Option<bool>,
	array_init_value_processing: HashMap<usize, Value>,
	is_global: bool,
	init_values: HashMap<i32, Value>,

	global_init_values: HashMap<String, Vec<InitValueItem>>,
}

impl Namer {
	pub fn transform(
		&mut self,
		program: &mut Program,
	) -> Result<HashMap<String, Vec<InitValueItem>>> {
		program.accept(self)?;
		Ok(self.global_init_values.to_owned())
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
			shirink(dim);
			dim_list.push(value.to_int()? as usize);
		}
		Ok(dim_list)
	}
}

impl Visitor for Namer {
	fn visit_program(&mut self, node: &mut Program) -> Result<()> {
		self.ctx.push();
		self.is_global = true;
		self.init_values.clear();
		for v in node.global_vars.iter_mut() {
			v.accept(self)?;
		}
		self.is_global = false;

		for v in node.functions.iter_mut() {
			v.accept(self)?;
		}
		self.ctx.pop()?;
		Ok(())
	}
	fn visit_func_decl(&mut self, node: &mut FuncDecl) -> Result<()> {
		self.ctx.push();
		let mut func_type = Vec::new();
		for param in node.formal_params.iter_mut() {
			param.accept(self)?;
			func_type.push(param.get_attr("type").unwrap().into());
		}
		let func_type: FuncType = (node.ret_type, func_type);
		let symbol = self.mgr.new_func_symbol(&node.ident, func_type);
		self.ctx.set_func(&node.ident, symbol)?;
		node.block.accept(self)?;
		self.ctx.pop()?;
		Ok(())
	}
	fn visit_var_def(&mut self, node: &mut VarDef) -> Result<()> {
		let dim_list = self.visit_dim_list(&mut node.dim_list)?;

		let mut align = vec![1];
		for item in dim_list.iter().rev() {
			align.push(align.last().unwrap() * item);
		}

		let (is_const, btype) = self.cur_type.unwrap();

		self.dim_list_processing = dim_list.clone().into();
		self.const_processing = is_const.into();
		self.alignment_processing = align.into();
		self.current_index_processing = 0;

		let var_type: VarType = (!is_const, btype, dim_list.clone()).into();

		let symbol = self.mgr.new_var_symbol(&node.ident, var_type);

		let symbol_id = symbol.id;
		let symbol_ident =
			symbol.ident.clone().split_whitespace().next().unwrap().to_string();
		node.set_attr("symbol", symbol.clone().into());

		self.ctx.set_val(&node.ident, symbol)?;
		if let Some(init) = node.init.as_mut() {
			init.mark_init_list_depth();
			self.array_init_value_processing.clear();
			init.accept(self)?;
			if is_const || self.is_global {
				if init.is_init_val_list() {
					let total_size =
						self.alignment_processing.as_ref().unwrap().last().unwrap();

					match btype {
						BType::Int => {
							let mut init_global_value = vec![];
							let mut next_global_index = 0;
							let mut typed_array_init_value = HashMap::new();

							for index in 0..*total_size {
								if let Some(value) =
									self.array_init_value_processing.get(&index)
								{
									let blank = index - next_global_index;
									if blank > 0 {
										init_global_value.push(InitValueItem::None(blank));
									}
									next_global_index = index + 1;
									match value {
										Value::Int(v) => {
											init_global_value.push(InitValueItem::Int(*v));
											typed_array_init_value.insert(index, *v);
										}
										Value::Float(v) => {
											init_global_value.push(InitValueItem::Int(*v as i32));
											typed_array_init_value.insert(index, *v as i32);
										}
										_ => {
											return Err(utils::SysycError::SyntaxError(
												"non scalar init value item for array!".to_string(),
											))
										}
									}
								}
							}

							let blank = total_size - next_global_index;
							if blank > 0 {
								init_global_value.push(InitValueItem::None(blank));
							}

							if is_const {
								self.init_values.insert(
									symbol_id,
									(dim_list.clone(), typed_array_init_value).into(),
								);
							}

							if self.is_global {
								self.global_init_values.insert(symbol_ident, init_global_value);
							}
						}
						BType::Float => {
							let mut init_global_value = vec![];
							let mut next_global_index = 0;
							let mut typed_array_init_value = HashMap::new();

							for index in 0..*total_size {
								if let Some(value) =
									self.array_init_value_processing.get(&index)
								{
									let blank = index - next_global_index;
									if blank > 0 {
										init_global_value.push(InitValueItem::None(blank));
									}
									next_global_index = index + 1;
									match value {
										Value::Int(v) => {
											init_global_value.push(InitValueItem::Float(*v as f32));
											typed_array_init_value.insert(index, *v as f32);
										}
										Value::Float(v) => {
											init_global_value.push(InitValueItem::Float(*v));
											typed_array_init_value.insert(index, *v);
										}
										_ => {
											return Err(utils::SysycError::SyntaxError(
												"non scalar init value item for array!".to_string(),
											))
										}
									}
								}
							}

							let blank = total_size - next_global_index;
							if blank > 0 {
								init_global_value.push(InitValueItem::None(blank));
							}

							if is_const {
								self.init_values.insert(
									symbol_id,
									(dim_list.clone(), typed_array_init_value).into(),
								);
							}

							if self.is_global {
								self.global_init_values.insert(symbol_ident, init_global_value);
							}
						}
					}
				} else {
					let init_value = init.get_attr("value");
					if let Some(attr::Attr::Value(value)) = init_value {
						match value {
							Value::Int(v) => match btype {
								BType::Int => {
									if self.is_global {
										self
											.global_init_values
											.insert(symbol_ident, vec![InitValueItem::Int(*v)]);
									}
									if is_const {
										self.init_values.insert(symbol_id, Value::Int(*v));
									}
								}
								BType::Float => {
									if self.is_global {
										self.global_init_values.insert(
											symbol_ident,
											vec![InitValueItem::Float(*v as f32)],
										);
									}
									if is_const {
										self.init_values.insert(symbol_id, Value::Float(*v as f32));
									}
								}
							},

							Value::Float(v) => match btype {
								BType::Int => {
									if self.is_global {
										self.global_init_values.insert(
											symbol_ident,
											vec![InitValueItem::Int(*v as i32)],
										);
									}
									if is_const {
										self.init_values.insert(symbol_id, Value::Int(*v as i32));
									}
								}
								BType::Float => {
									if self.is_global {
										self
											.global_init_values
											.insert(symbol_ident, vec![InitValueItem::Float(*v)]);
									}
									if is_const {
										self.init_values.insert(symbol_id, Value::Float(*v));
									}
								}
							},
							_ => {
								return Err(utils::SysycError::SyntaxError(
									"non scalar init value item for array!".to_string(),
								))
							}
						}
					}
				}
			}
		} else if self.is_global {
			// not initialized global
			match btype {
				BType::Int => {
					self
						.global_init_values
						.insert(symbol_ident, vec![InitValueItem::Int(0)]);
				}
				BType::Float => {
					self
						.global_init_values
						.insert(symbol_ident, vec![InitValueItem::Float(0.0)]);
				}
			}
		}
		self.dim_list_processing = None;
		self.const_processing = None;
		self.alignment_processing = None;

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
		if let Some(attr::Attr::InitValListDepth(depth)) =
			&node.get_attr("initvallistdepth")
		{
			let alignment =
				*(self.alignment_processing.as_ref().unwrap().get(*depth).unwrap());

			let odd = self.current_index_processing % alignment;
			if odd > 0 {
				self.current_index_processing += alignment - odd;
			}

			node.set_attr(
				"initvallist_index",
				attr::Attr::InitValLIstPosition(self.current_index_processing),
			);

			for val in node.val_list.iter_mut() {
				val.accept(self)?;
				if !val.is_init_val_list() {
					val.set_attr(
						"initvallist_index",
						attr::Attr::InitValLIstPosition(self.current_index_processing),
					);

					if let Some(attr::Attr::Value(value)) = val.get_attr("value") {
						self
							.array_init_value_processing
							.insert(self.current_index_processing, value.clone());
					} else if self.const_processing.unwrap() {
						return Err(utils::SysycError::SyntaxError(
							"non-const init value for const array!".to_string(),
						));
					}
					self.current_index_processing += 1;
				}
			}

			let odd = self.current_index_processing % alignment;
			if odd > 0 {
				self.current_index_processing += alignment - odd;
			}
		} else {
			unreachable!("depth should be calculated before visit init vallist")
		}

		Ok(())
	}
	fn visit_literal_int(&mut self, node: &mut LiteralInt) -> Result<()> {
		let value: Value = node.value.into();
		node.set_attr("value", value.into());
		Ok(())
	}
	fn visit_literal_float(&mut self, node: &mut LiteralFloat) -> Result<()> {
		let value: Value = node.value.into();
		node.set_attr("value", value.into());
		Ok(())
	}
	fn visit_binary_expr(&mut self, node: &mut BinaryExpr) -> Result<()> {
		node.lhs.accept(self)?;
		node.rhs.accept(self)?;
		shirink(&mut node.lhs);
		shirink(&mut node.rhs);
		if node.op != BinaryOp::Assign {
			let lhs = node.lhs.get_attr("value");
			let rhs = node.rhs.get_attr("value");
			if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
				let value = exec_binaryop(&lhs.into(), node.op, &rhs.into())?;
				node.set_attr("value", value.into());
			}
		} else if let Some(symbol) = node.lhs.get_attr("symbol") {
			node.set_attr("symbol", symbol.clone());
		}
		Ok(())
	}
	fn visit_unary_expr(&mut self, node: &mut UnaryExpr) -> Result<()> {
		node.rhs.accept(self)?;
		shirink(&mut node.rhs);
		if let Some(rhs) = node.rhs.get_attr("value") {
			let value = exec_unaryop(node.op, &rhs.into())?;
			node.set_attr("value", value.into());
		}
		Ok(())
	}
	fn visit_func_call(&mut self, node: &mut FuncCall) -> Result<()> {
		let symbol = self.ctx.find_func(&node.ident)?.clone();
		let v: Option<VarType> = symbol.var_type.0.into();
		if let Some(v) = v {
			node.set_attr("type", v.into());
		}
		node.set_attr("func_symbol", symbol.into());
		for param in node.params.iter_mut() {
			param.accept(self)?;
			shirink(param);
		}
		Ok(())
	}
	fn visit_formal_param(&mut self, node: &mut FormalParam) -> Result<()> {
		let dim_list = self.visit_dim_list(&mut node.dim_list)?;
		let var_type: VarType = (true, node.type_t, dim_list).into();
		let symbol = self.mgr.new_var_symbol(&node.ident, var_type.clone());
		node.set_attr("symbol", symbol.clone().into());
		self.ctx.set_val(&node.ident, symbol)?;
		node.set_attr("type", var_type.into());
		Ok(())
	}
	fn visit_variable(&mut self, node: &mut Variable) -> Result<()> {
		let symbol = self.ctx.find_val(&node.ident)?.clone();

		node.set_attr("symbol", symbol.clone().into());
		node.set_attr("type", symbol.var_type.into());

		if let Some(value) = self.init_values.get(&symbol.id) {
			node.set_attr("value", attr::Attr::Value(value.clone()));
		}

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
		shirink(&mut node.cond);
		node.body.accept(self)?;
		if let Some(then) = &mut node.then {
			then.accept(self)?;
		}
		Ok(())
	}
	fn visit_while(&mut self, node: &mut While) -> Result<()> {
		node.cond.accept(self)?;
		shirink(&mut node.cond);
		node.body.accept(self)
	}
	fn visit_continue(&mut self, _node: &mut Continue) -> Result<()> {
		Ok(())
	}
	fn visit_break(&mut self, _node: &mut Break) -> Result<()> {
		Ok(())
	}
	fn visit_return(&mut self, node: &mut Return) -> Result<()> {
		if let Some(val) = &mut node.value {
			val.accept(self)?;
			shirink(val);
		}
		Ok(())
	}
}
