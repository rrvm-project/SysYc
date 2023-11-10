use crate::utils::{
	array_init_for_backend, assert_is_convertable_to, DataFromNamer,
};
use ast::{tree::*, visitor::Visitor, BinaryOp, FuncType};
use attr::{Attr, Attrs, CompileConstValue, InitValueItem};
use ir_type::builtin_type::{BaseType, IRType};
use scope::{
	scope::ScopeStack,
	symbol::{FuncSymbol, VarSymbol},
};
use std::{collections::HashMap, vec};
use utils::SysycError;

use crate::complie_calculate::{
	evaluate_binary, evaluate_unary, type_binary, type_unary,
};

pub static COMPILE_CONST: &str = "compile_const";
pub static COMPILE_CONST_INDEX: &str = "compile_const_index";
pub static SYMBOL_NUMBER: &str = "symbol_number";
pub static TYPE: &str = "type";
pub static INDEX: &str = "index";

static _INIT_LIST_ALIGNMENT: &str = "init_list_alignment";
static _INIT_LIST_HEIGHT: &str = "init_list_height";

#[derive(Debug)]
struct InitListContext {
	pub dims_alignment: Vec<usize>,
	pub used_space: usize,
	pub init_values: HashMap<usize, InitValueItem>,
}
#[derive(Debug)]
struct VarDeclContext {
	pub vartype_when_visiting_var_declar: IRType,
}

#[derive(Debug)]
pub struct Namer {
	// namer 需要的context在这里存！
	pub loop_num: i32,

	pub scope_stack: ScopeStack,
	var_decl_context: Option<VarDeclContext>,
	init_list_context: Option<InitListContext>,
	pub var_symbols: Vec<VarSymbol>,
	pub func_symbols: Vec<FuncSymbol>,
}

impl Default for Namer {
	fn default() -> Self {
		Namer {
			loop_num: 0,
			scope_stack: ScopeStack::new(),
			var_decl_context: None,
			init_list_context: None,
			var_symbols: vec![],
			func_symbols: vec![],
		}
	}
}

impl Namer {
	pub fn transform(
		&mut self,
		mut program: Program,
	) -> Result<(Program, DataFromNamer), SysycError> {
		program.accept(self)?;

		let mut global_var_init_value =
			HashMap::<String, Vec<InitValueItem>>::new();
		for (name, symbol) in &self.scope_stack.current_scope().varsymbols {
			let value_to_backend = match &symbol.const_or_global_initial_value {
				Some(CompileConstValue::Int(value)) => vec![InitValueItem::Int(*value)],
				Some(CompileConstValue::Float(value)) => {
					vec![InitValueItem::Float(*value)]
				}
				Some(CompileConstValue::IntArray(value)) => {
					array_init_for_backend(value, |x| InitValueItem::Int(*x))
				}
				Some(CompileConstValue::FloatArray(value)) => {
					array_init_for_backend(value, |x| InitValueItem::Float(*x))
				}
				_ => vec![],
			};

			global_var_init_value.insert(name.to_string(), value_to_backend);
		}

		let data: DataFromNamer = DataFromNamer {
			global_var_init_value,
			var_symbols: std::mem::take(&mut self.var_symbols),
			func_symbols: std::mem::take(&mut self.func_symbols),
		};

		Ok((program, data))
	}
}

impl Visitor for Namer {
	fn visit_program(&mut self, program: &mut Program) -> Result<(), SysycError> {
		for unit in program.comp_units.iter_mut() {
			unit.accept(self)?;
		}

		if let Some(f) = &self.scope_stack.lookup_func("main") {
			if f.ret_t.base_type != BaseType::Int {
				return Err(SysycError::SyntaxError(
					"main function should return int".to_string(),
				));
			}
			if !f.params.is_empty() {
				return Err(SysycError::SyntaxError(
					"main function should not have any parameter".to_string(),
				));
			}
		} else {
			return Err(SysycError::SyntaxError(
				"main function not found".to_string(),
			));
		}
		Ok(())
	}

	fn visit_var_def(&mut self, val_def: &mut VarDef) -> Result<(), SysycError> {
		// val_def.init.as_mut().map(|init| init.accept(self));

		let mut var_type = match &mut self.var_decl_context {
			Some(var_not_finished) => {
				var_not_finished.vartype_when_visiting_var_declar.clone()
			}
			None => unreachable!("var_def must in a var_decl"),
		};

		let is_array = val_def.dim_list.is_some();
		let is_global = self.scope_stack.current_is_global();
		let mut init_value: Option<CompileConstValue> = None;

		let mut dim_list: Vec<usize> = vec![];

		for vec in val_def.dim_list.iter_mut() {
			for item in vec {
				item.accept(self)?;
				let value = item.get_attr(COMPILE_CONST);
				if let Some(Attr::CompileConstValue(CompileConstValue::Int(value))) =
					value
				{
					if value > &0 {
						dim_list.push(*value as usize);
						continue;
					}
				}
				return Err(SysycError::SyntaxError(
					"Illegal length for array".to_string(),
				));
			}
		}

		var_type.dims = dim_list;

		if is_array {
			let mut dims_alignment: Vec<usize> =
				Vec::with_capacity(var_type.dims.len());

			dims_alignment.push(1); // with height of 0!

			if let Some(value) = &mut val_def.init {
				for i in 1..(var_type.dims.len()) {
					let current_size = var_type.dims[var_type.dims.len() - i];
					dims_alignment.push(dims_alignment[i - 1] * current_size);
				}

				self.init_list_context = Some(InitListContext {
					dims_alignment,
					used_space: 0,
					init_values: HashMap::new(),
				});
				value.accept(self)?; // 维护每一个的alignment
				value.accept(self)?; // 具体分配index

				if is_global || var_type.is_const {
					init_value = match var_type.base_type {
						BaseType::Int => Some(CompileConstValue::IntArray(HashMap::new())),
						BaseType::Float => {
							Some(CompileConstValue::FloatArray(HashMap::new()))
						}
						_ => {
							return Err(SysycError::SyntaxError(format!(
								"{:?} is not a valid base type for array",
								var_type.base_type
							)))
						}
					};

					let init_value_from_context = std::mem::take(
						&mut self.init_list_context.as_mut().unwrap().init_values,
					);

					for (key, value) in init_value_from_context.into_iter() {
						match (var_type.base_type, &value) {
							(_, InitValueItem::None(_)) => {
								return Err(SysycError::SyntaxError(
									"Not compile time const in a global or const array"
										.to_string(),
								));
							}
							// TODO 优化逻辑？把2*2种代码搞成2+2种
							(BaseType::Int, InitValueItem::Int(value)) => {
								if let CompileConstValue::IntArray(table) =
									init_value.as_mut().unwrap()
								{
									table.insert(key, *value);
								}
							}
							(BaseType::Int, InitValueItem::Float(value)) => {
								if let CompileConstValue::IntArray(table) =
									init_value.as_mut().unwrap()
								{
									table.insert(key, *value as i32);
								}
							}
							(BaseType::Float, InitValueItem::Int(value)) => {
								if let CompileConstValue::FloatArray(table) =
									init_value.as_mut().unwrap()
								{
									table.insert(key, *value as f32);
								}
							}
							(BaseType::Float, InitValueItem::Float(value)) => {
								if let CompileConstValue::FloatArray(table) =
									init_value.as_mut().unwrap()
								{
									table.insert(key, *value);
								}
							}

							(_, _) => {
								return Err(SysycError::SyntaxError(format!(
									"{:?} found in a {:?} array",
									value, var_type.base_type
								)))
							}
						}
					}
				}

				self.init_list_context = None;
			}
		} else {
			println!("{:?}", var_type);
			init_value = match &mut val_def.init {
				Some(value) => {
					value.accept(self)?;

					if let Some(Attr::CompileConstValue(inner)) =
						value.get_attr(COMPILE_CONST)
					{
						match var_type.base_type {
							BaseType::Float => {
								Some(attr::CompileConstValue::Float(inner.to_f32()?))
							}
							BaseType::Int => {
								Some(attr::CompileConstValue::Int(inner.to_i32()?))
							}
							_ => None,
						}
					} else {
						None
					}
				}

				None => None,
			};
		}

		let symbol = VarSymbol {
			name: val_def.ident.clone(),
			tp: var_type.clone(),
			is_global: self.scope_stack.current_is_global(),
			id: self.var_symbols.len(),
			const_or_global_initial_value: init_value,
		};

		self.scope_stack.declare_var(&symbol);

		self.var_symbols.push(symbol);

		Ok(())
	}
	fn visit_var_decl(
		&mut self,
		val_decl: &mut VarDecl,
	) -> Result<(), SysycError> {
		let var_type = IRType {
			base_type: match &val_decl.type_t {
				ast::VarType::Int => BaseType::Int,
				ast::VarType::Float => BaseType::Float,
			},
			dims: vec![],
			is_const: val_decl.is_const,
		};

		self.var_decl_context = Some(VarDeclContext {
			vartype_when_visiting_var_declar: var_type,
		});

		for def in val_decl.defs.iter_mut() {
			def.accept(self)?;
		}

		self.var_decl_context = None;

		Ok(())
	}
	fn visit_func_decl(
		&mut self,
		func_decl: &mut FuncDecl,
	) -> Result<(), SysycError> {
		if self
			.scope_stack
			.current_scope()
			.lookup_func(func_decl.ident.as_str())
			.is_some()
			|| self
				.scope_stack
				.current_scope()
				.lookup_var(func_decl.ident.as_str())
				.is_some()
		{
			return Err(SysycError::SyntaxError(format!(
				"Identifier {} already declared",
				func_decl.ident
			)));
		}

		if !self.scope_stack.current_is_global() {
			return Err(SysycError::SyntaxError(format!(
				"Function {} is not defined globally",
				func_decl.ident
			)));
		}

		self.scope_stack.push();

		let mut params = vec![];

		for param in func_decl.formal_params.iter_mut() {
			param.accept(self)?;
			if let Some(Attr::Type(param_type)) = param.get_attr(TYPE) {
				params.push(param_type.clone())
			} else {
				unreachable!("formal param must have a type")
			}
		}

		let symbol = FuncSymbol {
			name: func_decl.ident.clone(),
			// ret_t: func_decl.func_type.clone(),
			ret_t: match func_decl.func_type {
				FuncType::Float => IRType::get_scalar(BaseType::Float, false),
				FuncType::Int => IRType::get_scalar(BaseType::Int, false),
				FuncType::Void => IRType::get_scalar(BaseType::Void, false),
			},
			params,
			id: self.func_symbols.len(),
		};

		self.scope_stack.declare_func_at_global(&symbol);
		self.func_symbols.push(symbol);

		func_decl.block.accept(self)?;

		self.scope_stack.pop();
		Ok(())
	}
	fn visit_init_val_list(
		&mut self,
		val_list: &mut InitValList,
	) -> Result<(), SysycError> {
		if self.init_list_context.is_none() {
			unreachable!("init list must in a var_decl of array");
		};

		let first_pass = val_list.get_attr(_INIT_LIST_HEIGHT).is_none();

		if first_pass {
			let mut max_depth_of_child: usize = 0;
			for item in &mut val_list.val_list {
				item.accept(self)?;
				if let Some(Attr::UIntValue(height)) = item.get_attr(_INIT_LIST_HEIGHT)
				{
					if max_depth_of_child < *height {
						max_depth_of_child = *height;
					}
				} else {
					item.set_attr(_INIT_LIST_HEIGHT, Attr::UIntValue(0));
				}
				// val_list.set_attr(, attr)
			}
			val_list
				.set_attr(_INIT_LIST_HEIGHT, Attr::UIntValue(max_depth_of_child + 1));
			Ok(())
		} else {
			for item in &mut val_list.val_list {
				if let Some(Attr::UIntValue(height)) = item.get_attr(_INIT_LIST_HEIGHT)
				{
					let alignment =
						self.init_list_context.as_ref().unwrap().dims_alignment[*height];
					let used = self.init_list_context.as_ref().unwrap().used_space;
					let blank = used % alignment;
					let position = used + if blank == 0 { 0 } else { alignment - blank };
					let height = *height;
					if height == 0 {
						self.init_list_context.as_mut().unwrap().used_space =
							position + alignment;
						self.init_list_context.as_mut().unwrap().init_values.insert(
							position,
							match item.get_attr(COMPILE_CONST) {
								Some(Attr::CompileConstValue(CompileConstValue::Int(
									value,
								))) => InitValueItem::Int(*value),
								Some(Attr::CompileConstValue(CompileConstValue::Float(
									value,
								))) => InitValueItem::Float(*value),
								_ => InitValueItem::None(1),
							},
						);
					} else {
						self.init_list_context.as_mut().unwrap().used_space = position;
					}
					item.set_attr(INDEX, Attr::UIntValue(position));

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
	fn visit_literal_int(
		&mut self,
		literal_int: &mut LiteralInt,
	) -> Result<(), SysycError> {
		literal_int.set_attr(
			COMPILE_CONST,
			attr::Attr::CompileConstValue(CompileConstValue::Int(literal_int.value)),
		);
		literal_int.set_attr(
			TYPE,
			attr::Attr::Type(IRType {
				base_type: BaseType::Int,
				dims: vec![],
				is_const: false,
			}),
		);
		Ok(())
	}

	fn visit_literal_float(
		&mut self,
		literal_float: &mut LiteralFloat,
	) -> Result<(), SysycError> {
		literal_float.set_attr(
			COMPILE_CONST,
			attr::Attr::CompileConstValue(CompileConstValue::Float(
				literal_float.value,
			)),
		);
		literal_float.set_attr(
			TYPE,
			attr::Attr::Type(IRType {
				base_type: BaseType::Float,
				dims: vec![],
				is_const: false,
			}),
		);
		Ok(())
	}
	fn visit_binary_expr(
		&mut self,
		binary_expr: &mut BinaryExpr,
	) -> Result<(), SysycError> {
		binary_expr.rhs.accept(self)?;
		binary_expr.lhs.accept(self)?;

		// typer
		if let (Some(Attr::Type(type_l)), Some(Attr::Type(type_r))) = (
			binary_expr.lhs.get_attr(TYPE),
			binary_expr.rhs.get_attr(TYPE),
		) {
			binary_expr.set_attr(
				TYPE,
				Attr::Type(type_binary(type_l, &binary_expr.op, type_r)?),
			);
		} else {
			return Err(SysycError::SyntaxError(format!(
				"Failed to determain type of the assignment {:?}",
				binary_expr
			)));
		}

		// compile const value
		if binary_expr.op == BinaryOp::Assign {
			// 只要赋值语句的右边是常量，结果就是常量

			if let Some(r) = binary_expr.rhs.get_attr(COMPILE_CONST) {
				binary_expr.set_attr(COMPILE_CONST, r.clone());
			}
		} else {
			let r = binary_expr.rhs.get_attr(COMPILE_CONST);
			let l = binary_expr.lhs.get_attr(COMPILE_CONST);
			if r.is_some() && l.is_some() {
				if let (Attr::CompileConstValue(l), Attr::CompileConstValue(r)) =
					(l.unwrap(), r.unwrap())
				{
					let result = evaluate_binary(l, &binary_expr.op, r)?;
					binary_expr.set_attr(COMPILE_CONST, Attr::CompileConstValue(result));
				}
			}
		}

		Ok(())
	}
	fn visit_unary_expr(
		&mut self,
		unary_expr: &mut UnaryExpr,
	) -> Result<(), SysycError> {
		unary_expr.rhs.accept(self)?;

		// typer
		if let Some(Attr::Type(type_r)) = unary_expr.rhs.get_attr(TYPE) {
			unary_expr
				.set_attr(TYPE, Attr::Type(type_unary(&unary_expr.op, type_r)?));
		} else {
			return Err(SysycError::SyntaxError(format!(
				"Failed to determain type of the assignment {:?}",
				unary_expr
			)));
		}

		//compile const value
		if let Some(Attr::CompileConstValue(const_value)) =
			unary_expr.rhs.get_attr(COMPILE_CONST)
		{
			unary_expr.set_attr(
				COMPILE_CONST,
				Attr::CompileConstValue(evaluate_unary(&unary_expr.op, const_value)?),
			);
		}

		Ok(())
	}
	fn visit_func_call(
		&mut self,
		func_call: &mut FuncCall,
	) -> Result<(), SysycError> {
		if let Some(func_symbol) = self.scope_stack.lookup_func(&func_call.ident) {
			// 给FuncCall绑定上一个FuncSymbol
			func_call.set_attr(SYMBOL_NUMBER, Attr::FuncSymbol(func_symbol.id));
			if func_call.params.len() != func_symbol.params.len() {
				return Err(SysycError::SyntaxError(format!(
					"In correct argument list length when calling {}",
					func_symbol.name
				)));
			}

			let target_params = func_symbol.params.clone();

			for (i, item) in func_call.params.iter_mut().enumerate() {
				item.accept(self)?;
				if let Some(Attr::Type(argument_type)) = item.get_attr(TYPE) {
					assert_is_convertable_to(argument_type, &target_params[i])?;
				}
			}
		} else {
			return Err(SysycError::SyntaxError(format!(
				"Unknown func symbol {:?}",
				func_call.ident
			)));
		}
		Ok(())
	}

	fn visit_formal_param(
		&mut self,
		formal_param: &mut FormalParam,
	) -> Result<(), SysycError> {
		let base_type = match formal_param.type_t {
			ast::VarType::Int => BaseType::Int,
			ast::VarType::Float => BaseType::Float,
		};

		let mut dim_list = vec![];

		if let Some(items) = &mut formal_param.dim_list {
			dim_list.push(0); // 形式参数数组的第0维当作是0
			for item in items {
				item.accept(self)?;
				if let Some(Attr::CompileConstValue(CompileConstValue::Int(value))) =
					item.get_attr(COMPILE_CONST)
				{
					if *value < 0 {
						return Err(SysycError::SyntaxError(format!(
							"Error in {}'s dim list. Need to be non-negative",
							formal_param.ident
						)));
					}
					dim_list.push(*value as usize);
				} else {
					return Err(SysycError::SyntaxError(format!("Error in {}'s dim list. Only compile time const integer is allowed",formal_param.ident )));
				}
			}
		}

		let param_type = IRType {
			base_type,
			dims: dim_list,
			is_const: false,
		};
		formal_param.set_attr(TYPE, Attr::Type(param_type.clone()));

		let symbol = VarSymbol {
			id: self.var_symbols.len(),
			name: formal_param.ident.clone(),
			is_global: false,
			tp: param_type,
			const_or_global_initial_value: None,
		};

		self.scope_stack.declare_var(&symbol);

		formal_param.set_attr(SYMBOL_NUMBER, Attr::VarSymbol(symbol.id));

		self.var_symbols.push(symbol);

		Ok(())
	}
	fn visit_lval(&mut self, lval: &mut Lval) -> Result<(), SysycError> {
		let symbol = self.scope_stack.lookup_var(&lval.ident);
		if symbol.is_none() {
			return Err(SysycError::SyntaxError(
				"undefined variable ".to_string() + &lval.ident,
			));
		}
		let symbol = symbol.unwrap();
		let symbol_id = symbol.id;

		lval.set_attr(SYMBOL_NUMBER, Attr::VarSymbol(symbol.id));

		if symbol.tp.is_array() && lval.dim_list.is_some() {
			let mut reduced_type = symbol.tp.clone();
			reduced_type.dims = vec![];
			//如果维度对不上，会在下面报错！
			lval.set_attr(TYPE, Attr::Type(reduced_type));
		} else {
			lval.set_attr(TYPE, Attr::Type(symbol.tp.clone()));
		}

		// HACK : 目前来看可行。但是没有把[]处理成取下标运算的做法是没有可扩展性的
		if symbol.tp.is_array() {
			if let Some(dim_list) = lval.dim_list.as_mut() {
				if dim_list.len() != symbol.tp.dims.len() {
					return Err(SysycError::SyntaxError(format!(
						"{:?} has {:?} dims, not {:?} dims",
						symbol.name,
						symbol.tp.dims.len(),
						dim_list.len()
					)));
				}

				let mut dim_item_const_value: Vec<usize> = vec![];
				let mut dim_all_const = true;

				for dim_item in dim_list {
					dim_item.accept(self)?;

					if let Some(Attr::Type(type_of_index)) = dim_item.get_attr(TYPE) {
						if !type_of_index.is_scalar() {
							return Err(SysycError::SyntaxError(
								"Only scalar types are accepted as indexes of array!"
									.to_string(),
							));
						}
					}

					if let Some(Attr::CompileConstValue(compile_value)) =
						dim_item.get_attr(COMPILE_CONST)
					{
						let index = match compile_value {
							CompileConstValue::Int(value) => *value,
							CompileConstValue::Float(value) => *value as i32,
							_ => unreachable!(),
						};
						if index < 0 {
							return Err(SysycError::SyntaxError(
								"Index of array must be non-negative!".to_string(),
							));
						}
						dim_item_const_value.push(index as usize)
					} else {
						dim_all_const = false;
					}
				}

				let symbol = &self.var_symbols[symbol_id];
				if dim_all_const {
					let index = symbol.tp.get_index(&dim_item_const_value);
					lval.set_attr(COMPILE_CONST_INDEX, Attr::UIntValue(index));
					if symbol.tp.is_const {
						if let Some(value) = &symbol.const_or_global_initial_value {
							match value {
								CompileConstValue::IntArray(array) => {
									if let Some(indexed_value) = array.get(&index) {
										lval.set_attr(
											COMPILE_CONST,
											Attr::CompileConstValue(CompileConstValue::Int(
												*indexed_value,
											)),
										)
									} else {
										lval.set_attr(
											COMPILE_CONST,
											Attr::CompileConstValue(CompileConstValue::Int(0)),
										)
									};
								}
								CompileConstValue::FloatArray(array) => {
									if let Some(indexed_value) = array.get(&index) {
										lval.set_attr(
											COMPILE_CONST,
											Attr::CompileConstValue(CompileConstValue::Float(
												*indexed_value,
											)),
										)
									} else {
										lval.set_attr(
											COMPILE_CONST,
											Attr::CompileConstValue(CompileConstValue::Float(0.0)),
										)
									};
								}
								_ => {
									return Err(SysycError::SyntaxError(
										"scalar type can not be indexed".to_string(),
									))
								}
							}
						}
					}
				}
			}
		// It is legal when dim_list is none, such as passing the array as an argument
		} else if let Some(value) = &symbol.const_or_global_initial_value {
			lval.set_attr(COMPILE_CONST, Attr::CompileConstValue(value.clone()));
		}

		Ok(())
	}
	fn visit_block(&mut self, block: &mut Block) -> Result<(), SysycError> {
		self.scope_stack.push();

		for statement in &mut block.stmts {
			statement.accept(self)?;
		}

		self.scope_stack.pop();
		Ok(())
	}
	fn visit_if(&mut self, if_statement: &mut If) -> Result<(), SysycError> {
		if_statement.cond.accept(self)?;
		if_statement.body.accept(self)?;
		Ok(())
	}
	fn visit_while(
		&mut self,
		while_statement: &mut While,
	) -> Result<(), SysycError> {
		while_statement.cond.accept(self)?;
		while_statement.body.accept(self)?;
		Ok(())
	}
	fn visit_continue(
		&mut self,
		_continue: &mut Continue,
	) -> Result<(), SysycError> {
		Ok(())
	}
	fn visit_break(&mut self, _break: &mut Break) -> Result<(), SysycError> {
		Ok(())
	}
	fn visit_return(
		&mut self,
		return_statement: &mut Return,
	) -> Result<(), SysycError> {
		if let Some(return_value) = &mut return_statement.value {
			return_value.accept(self)?;
		}
		Ok(())
	}
}
