use ast::{tree::*, visitor::Visitor, BinaryOp, FuncType};
use ir_type::builtin_type::{BaseType, IRType};
use scope::{
	scope::ScopeStack,
	symbol::{self, FuncSymbol, VarSymbol},
};
use std::{collections::HashMap, vec};
use utils::{Attr, Attrs, CompileConstValue, InitValueItem, SysycError};

use crate::complie_calculate::evaluate_binary;

static COMPILE_CONST: &'static str = "compile_const";
static SYMBOL_NUMBER: &'static str = "symbol_number";

#[derive(Debug)]
pub struct Namer {
	// namer 需要的context在这里存！
	pub loop_num: i32,
	pub scope_stack: ScopeStack,
	pub vartype_when_visiting_var_declar: Option<IRType>,
	pub global_var: HashMap<String, Vec<InitValueItem>>,
	pub const_var: HashMap<String, CompileConstValue>,
	pub var_symbols: Vec<VarSymbol>,
	pub func_symbols: Vec<FuncSymbol>,
}

impl Default for Namer {
	fn default() -> Self {
		Namer {
			loop_num: 0,
			scope_stack: ScopeStack::new(),
			vartype_when_visiting_var_declar: None,
			global_var: HashMap::new(),
			const_var: HashMap::new(),
			var_symbols: vec![],
			func_symbols: vec![],
		}
	}
}

impl Namer {
	pub fn transform(
		&mut self,
		mut program: Program,
	) -> Result<Program, SysycError> {
		program.accept(self)?;
		// 给每个ident，对应根据可见性规则的VarSymbol或者FuncSymbol
		Ok(program)
	}
}

impl Visitor for Namer {
	fn visit_program(&mut self, program: &mut Program) -> Result<(), SysycError> {
		for unit in program.comp_units.iter_mut() {
			unit.accept(self)?;
		}

		if let Some(f) = &self.scope_stack.lookup_func("main") {
			if f.ret_t != BaseType::Int {
				return Err(SysycError::SyntaxError(
					"main function should return int".to_string(),
				));
			}
			if f.param_num() != 0 {
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

		let mut var_type = match &self.vartype_when_visiting_var_declar {
			Some(var_not_finished) => var_not_finished.clone(),
			None => unreachable!("var_def must in a var_decl"),
		};

		let is_array = val_def.dim_list.is_some();

		if is_array {
			match &mut val_def.init {
				Some(value) => {
					value.accept(self)?;
					println!("{:?}", value);
				}
				None => (),
			};
		// TODO : calculate init for array!
		} else {
			let init_value: Option<CompileConstValue> = match &mut val_def.init {
				Some(value) => {
					value.accept(self)?;
					match value.get_attr(COMPILE_CONST) {
						Some(comp_value) => match comp_value {
							Attr::CompileConstValue(inner) => Some(inner.clone()),
							_ => None,
						},
						None => None,
					}
				}
				None => None,
			};

			if init_value.is_some() && var_type.is_const {
				self
					.const_var
					.insert(val_def.ident.clone(), init_value.clone().unwrap());
			}

			if self.scope_stack.current_is_global() {
				if let Some(value) = init_value {
					match value {
						CompileConstValue::Int(inner) => {
							self.global_var.insert(
								val_def.ident.clone().to_string(),
								vec![InitValueItem::Int(inner)],
							);
						}
						CompileConstValue::Float(inner) => {
							self.global_var.insert(
								val_def.ident.clone().to_string(),
								vec![InitValueItem::Float(inner)],
							);
						}
						_ => {
							return Err(SysycError::SyntaxError(
								"未知全局变量类型".to_string(),
							));
						}
					}
				} else {
					self.global_var.insert(val_def.ident.clone().to_string(), vec![]);
				}
			}
		}

		let mut dim_list: Vec<usize> = vec![];

		for vec in val_def.dim_list.iter_mut() {
			for item in vec {
				item.accept(self)?;
				println!("item in dim list {:?}", item);
				println!("const value for item{:?}", item.get_attr(COMPILE_CONST));
				let value = item.get_attr(COMPILE_CONST);
				if let Some(value) = value {
					if let Attr::CompileConstValue(CompileConstValue::Int(value)) = value{
						if value > &0 {
							dim_list.push(*value as usize);
							continue;
						}
					}
				} 
				return  Err(SysycError::SyntaxError("Illegal length for array".to_string()));
			}
		}

		var_type.dims = dim_list;

		let symbol = VarSymbol {
			name: val_def.ident.clone(),
			tp: var_type,
			is_global: self.scope_stack.current_is_global(),
			id: self.var_symbols.len(),
		};

		self.scope_stack.declare_var(&symbol);

		self.var_symbols.push(symbol);

		println!("current car_symbols{:?}", self.var_symbols);

		println!("current global vars{:?}", self.global_var);

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

		self.vartype_when_visiting_var_declar = Some(var_type);

		for def in val_decl.defs.iter_mut() {
			def.accept(self)?;
		}

		self.vartype_when_visiting_var_declar = None;

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
		let symbol = FuncSymbol {
			name: func_decl.ident.clone(),
			// ret_t: func_decl.func_type.clone(),
			ret_t: match func_decl.func_type {
				FuncType::Float => BaseType::Float,
				FuncType::Int => BaseType::Int,
				FuncType::Void => BaseType::Void,
			},
			params: vec![],
			id: self.func_symbols.len(),
		};

		self.scope_stack.declare_func(&symbol);

		self.func_symbols.push(symbol);

		self.scope_stack.push();

		for param in func_decl.formal_params.iter_mut() {
			param.accept(self)?;
		}

		func_decl.block.accept(self)?;

		self.scope_stack.pop();
		Ok(())
	}
	fn visit_init_val_list(
		&mut self,
		val_list: &mut InitValList,
	) -> Result<(), SysycError> {
		todo!()
	}
	fn visit_literal_int(
		&mut self,
		literal_int: &mut LiteralInt,
	) -> Result<(), SysycError> {
		literal_int.set_attr(
			COMPILE_CONST,
			utils::Attr::CompileConstValue(CompileConstValue::Int(literal_int.value)),
		);
		Ok(())
	}
	fn visit_literal_float(
		&mut self,
		literal_float: &mut LiteralFloat,
	) -> Result<(), SysycError> {
		literal_float.set_attr(
			COMPILE_CONST,
			utils::Attr::CompileConstValue(CompileConstValue::Float(
				literal_float.value,
			)),
		);
		Ok(())
	}
	fn visit_binary_expr(
		&mut self,
		binary_expr: &mut BinaryExpr,
	) -> Result<(), SysycError> {
		binary_expr.rhs.accept(self)?;
		binary_expr.lhs.accept(self)?;

		if binary_expr.op == BinaryOp::Assign {
			let r = binary_expr.rhs.get_attr(COMPILE_CONST);
			if r.is_some() {
				binary_expr.lhs.set_attr(COMPILE_CONST, r.unwrap().clone());
				binary_expr.set_attr(COMPILE_CONST, r.unwrap().clone());
			}
		// TODO 检查赋值的合法性
		} else {
			let r = binary_expr.rhs.get_attr(COMPILE_CONST);
			let l = binary_expr.lhs.get_attr(COMPILE_CONST);
			if r.is_some() && l.is_some() {
				if let (Attr::CompileConstValue(l), Attr::CompileConstValue(r)) =
					(l.unwrap(), r.unwrap())
				{
					let result = evaluate_binary(r, &binary_expr.op, l)?;
					binary_expr.set_attr(COMPILE_CONST, Attr::CompileConstValue(result));
					println!("calculated const {:?}", binary_expr.get_attr(COMPILE_CONST));
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
		!todo!();

		Ok(())
	}
	fn visit_func_call(
		&mut self,
		val_decl: &mut FuncCall,
	) -> Result<(), SysycError> {
		todo!()
	}
	fn visit_formal_param(
		&mut self,
		val_decl: &mut FormalParam,
	) -> Result<(), SysycError> {
		todo!()
	}
	fn visit_lval(&mut self, lval: &mut Lval) -> Result<(), SysycError> {
		let symbol = self.scope_stack.lookup_var(&lval.ident);
		if symbol.is_none() {
			return Err(SysycError::SyntaxError(
				"undefined variable ".to_string() + &lval.ident,
			));
		}
		let symbol = symbol.unwrap();

		lval.set_attr(SYMBOL_NUMBER, Attr::Symbol(symbol.id));

		match self.const_var.get(&symbol.name) {
			Some(value) => {
				lval.set_attr(COMPILE_CONST, Attr::CompileConstValue(value.clone()));
			}
			None => (),
		};

		Ok(())
	}
	fn visit_block(&mut self, val_decl: &mut Block) -> Result<(), SysycError> {
		todo!()
	}
	fn visit_if(&mut self, val_decl: &mut If) -> Result<(), SysycError> {
		todo!()
	}
	fn visit_while(&mut self, val_decl: &mut While) -> Result<(), SysycError> {
		todo!()
	}
	fn visit_continue(
		&mut self,
		val_decl: &mut Continue,
	) -> Result<(), SysycError> {
		todo!()
	}
	fn visit_break(&mut self, val_decl: &mut Break) -> Result<(), SysycError> {
		todo!()
	}
	fn visit_return(&mut self, val_decl: &mut Return) -> Result<(), SysycError> {
		todo!()
	}
}
