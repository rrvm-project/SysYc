use std::collections::HashMap;

use ast::{tree::*, visitor::Visitor};
use attr::{Attr, Attrs};
use llvm::{
	basicblock::BasicBlock,
	func::LlvmFunc,
	llvmfuncemitter::LlvmFuncEmitter,
	llvmop::{ConvertOp, Value},
	llvmvar::VarType,
	LlvmProgram, Temp,
};
use rrvm_symbol::VarSymbol;
// use namer::{
// 	namer::{COMPILE_CONST, COMPILE_CONST_INDEX, INDEX, SYMBOL_NUMBER},
// 	utils::DataFromNamer,
// };
use utils::{errors::Result, InitValueItem, Label, SysycError};
use value::{BType, FuncRetType};

static VALUE: &str = "value";

// 意思是 irgen 过程中会给节点挂上的 attribute，内容是llvm::llvmop::Value, 名字可能名不副实
static IRVALUE: &str = "irvalue";
static FUNC_SYMBOL: &str = "func_symbol";
static SYMBOL: &str = "symbol";
static CUR_SYMBOL: &str = "cur_symbol";
// 数组初始化列表中每一项在数组中的位置
static INDEX: &str = "init_value_index";
static GLOBAL_VALUE: &str = "global_value";

pub struct LlvmIrGen {
	pub funcemitter: Option<LlvmFuncEmitter>,
	pub funcs: Vec<LlvmFunc>,
	pub global_vars: HashMap<Temp, Vec<InitValueItem>>,
	// pub data: DataFromNamer,
}

impl LlvmIrGen {
	pub fn transform(&mut self, mut program: Program) -> Result<()> {
		program.accept(self)
	}
	pub fn emit_program(self) -> LlvmProgram {
		LlvmProgram {
			funcs: self.funcs,
			global_vars: self.global_vars,
		}
	}
	fn convert(&mut self, to_type: VarType, value: Value) -> Value {
		if value.get_type() == to_type {
			return value;
		}
		match &value {
			Value::Float(f) => {
				if to_type == VarType::I32 {
					Value::Int(*f as i32)
				} else {
					unreachable!("Float can not be converted to ptr or void")
				}
			}
			Value::Int(i) => {
				if to_type == VarType::F32 {
					Value::Float(*i as f32)
				} else {
					unreachable!("Int can not be converted to ptr or void")
				}
			}
			Value::Temp(t) => {
				let op = match t.var_type {
					VarType::I32 => {
						if to_type == VarType::F32 {
							ConvertOp::Int2Float
						} else {
							unreachable!("Int can not be converted to ptr or void")
						}
					}
					VarType::F32 => {
						if to_type == VarType::I32 {
							ConvertOp::Float2Int
						} else {
							unreachable!("Float can not be converted to ptr or void")
						}
					}
					_ => unreachable!(),
				};
				Value::Temp(
					self
						.funcemitter
						.as_mut()
						.unwrap()
						.visit_convert_instr(op, t.var_type, value, to_type),
				)
			}
		}
	}
	fn visit_global_def(&mut self, val_decl: &mut VarDef) -> Result<()> {
		if let Some(Attr::VarSymbol(symbol)) = val_decl.get_attr(SYMBOL) {
			let tp = match (symbol.var_type.1, symbol.var_type.2.len()) {
				(BType::Int, 0) => llvm::llvmvar::VarType::I32,
				(BType::Float, 0) => llvm::llvmvar::VarType::F32,
				(BType::Int, _) => llvm::llvmvar::VarType::I32Ptr,
				(BType::Float, _) => llvm::llvmvar::VarType::F32Ptr,
			};
			self.global_vars.insert(
				Temp::new_global(val_decl.ident.clone(), tp),
				match val_decl.get_attr(GLOBAL_VALUE) {
					Some(Attr::GlobalValue(v)) => v.to_owned(),
					_ => unreachable!(),
				},
			);
		}
		Ok(())
	}
}

impl Visitor for LlvmIrGen {
	fn visit_program(&mut self, program: &mut Program) -> Result<()> {
		for comp_unit in &mut program.comp_units {
			comp_unit.accept(self)?;
		}
		Ok(())
	}
	fn visit_func_decl(&mut self, val_decl: &mut FuncDecl) -> Result<()> {
		let ret_type = match val_decl.ret_type {
			FuncRetType::Int => llvm::llvmvar::VarType::I32,
			FuncRetType::Float => llvm::llvmvar::VarType::F32,
			FuncRetType::Void => llvm::llvmvar::VarType::Void,
		};

		let entry = BasicBlock::new(0, Label::new("entry"), vec![]);
		let exit = BasicBlock::new(1, Label::new("exit"), vec![]);

		self.funcemitter = Some(LlvmFuncEmitter::new(
			val_decl.ident.clone(),
			ret_type,
			vec![],
			entry,
			exit,
		));

		for param in &mut val_decl.formal_params {
			param.accept(self)?;
		}

		val_decl.block.accept(self)?;
		self.funcs.push(self.funcemitter.take().unwrap().visit_end());
		self.funcemitter = None;
		Ok(())
	}
	fn visit_var_decl(&mut self, val_decl: &mut VarDecl) -> Result<()> {
		for def in val_decl.defs.iter_mut() {
			def.accept(self)?;
		}
		Ok(())
	}
	fn visit_var_def(&mut self, val_decl: &mut VarDef) -> Result<()> {
		if self.funcemitter.is_none() {
			self.visit_global_def(val_decl)?;
			return Ok(());
		}
		let symbol: VarSymbol = val_decl
			.get_attr(SYMBOL)
			.ok_or_else(|| {
				SysycError::LlvmSyntexError(format!(
					"var def {} has no symbol",
					val_decl.ident.clone()
				))
			})?
			.into();

		let var_type = symbol.var_type.clone();
		// 分配空间与初始化
		if !var_type.2.is_empty() {
			// 是数组
			let tp = match var_type.1 {
				BType::Int => llvm::llvmvar::VarType::I32Ptr,
				BType::Float => llvm::llvmvar::VarType::F32Ptr,
			};
			// TODO: 这里的Value里面应该是个usize还是个i32呢
			let mut size = 4;
			for i in var_type.2 {
				size *= i as i32;
			}
			let temp = self
				.funcemitter
				.as_mut()
				.unwrap()
				.visit_alloc_instr(tp, Value::Int(size));
			self
				.funcemitter
				.as_mut()
				.unwrap()
				.get_cur_basicblock()
				.symbol2temp
				.insert(symbol.id as usize, temp);
			// 初始化
			if let Some(init) = &mut val_decl.init {
				// 这里应当是一个初始化列表，设置一个Attr告知正在对哪个数组做初始化
				init.set_attr(CUR_SYMBOL, Attr::VarSymbol(symbol.clone()));
				init.accept(self)?;
			}
		} else {
			// 是标量
			let tp = match var_type.1 {
				BType::Int => llvm::llvmvar::VarType::I32,
				BType::Float => llvm::llvmvar::VarType::F32,
			};
			// 初始化
			let temp = self.funcemitter.as_mut().unwrap().fresh_temp(tp);
			self
				.funcemitter
				.as_mut()
				.unwrap()
				.get_cur_basicblock()
				.symbol2temp
				.insert(symbol.id as usize, temp.clone());
			if let Some(init) = &mut val_decl.init {
				init.accept(self)?;
				if let Some(Attr::Value(const_value)) = init.get_attr(VALUE) {
					match const_value {
						value::Value::Int(v) => {
							self
								.funcemitter
								.as_mut()
								.unwrap()
								.visit_assign_instr(temp, Value::Int(*v));
						}
						value::Value::Float(v) => {
							self
								.funcemitter
								.as_mut()
								.unwrap()
								.visit_assign_instr(temp, Value::Float(*v));
						}
						_ => {
							return Err(SysycError::LlvmSyntexError(format!(
								"const value for {} should not be other than float and int",
								val_decl.ident.clone()
							)))
						}
					}
				} else {
					let init_value = match init.get_attr(IRVALUE) {
						Some(Attr::IRValue(v)) => v.clone(),
						_ => {
							return Err(SysycError::LlvmSyntexError(format!(
								"init value for {} has no value",
								val_decl.ident.clone()
							)))
						}
					};
					self
						.funcemitter
						.as_mut()
						.unwrap()
						.visit_assign_instr(temp, init_value);
				}
			}
		}
		Ok(())
	}
	fn visit_formal_param(&mut self, val_decl: &mut FormalParam) -> Result<()> {
		let var_type = match (val_decl.type_t, val_decl.dim_list.len()) {
			(BType::Int, 0) => llvm::llvmvar::VarType::I32,
			(BType::Float, 0) => llvm::llvmvar::VarType::F32,
			(BType::Int, _) => llvm::llvmvar::VarType::I32Ptr,
			(BType::Float, _) => llvm::llvmvar::VarType::F32Ptr,
		};
		let tmp = self.funcemitter.as_mut().unwrap().visit_formal_param(var_type);
		match val_decl.get_attr(SYMBOL) {
			Some(Attr::VarSymbol(symbol)) => self
				.funcemitter
				.as_mut()
				.unwrap()
				.get_cur_basicblock()
				.symbol2temp
				.insert(symbol.id as usize, tmp),
			_ => {
				return Err(SysycError::LlvmSyntexError(format!(
					"param {} has no symbol",
					val_decl.ident.clone()
				)))
			}
		};
		Ok(())
	}
	fn visit_block(&mut self, val_decl: &mut Block) -> Result<(), SysycError> {
		for stmt in &mut val_decl.stmts {
			stmt.accept(self)?;
		}
		Ok(())
	}
	fn visit_func_call(&mut self, val_decl: &mut FuncCall) -> Result<()> {
		let funcsymbol = match val_decl.get_attr(FUNC_SYMBOL) {
			Some(Attr::FuncSymbol(symbol)) => symbol.clone(),
			_ => {
				return Err(SysycError::LlvmSyntexError(format!(
					"call {} has no funcsymbol",
					val_decl.ident.clone()
				)))
			}
		};
		let mut params = vec![];
		for (param, para_type) in
			val_decl.params.iter_mut().zip(funcsymbol.var_type.1)
		{
			param.accept(self)?;
			if let Some(Attr::Value(const_value)) = param.get_attr(VALUE) {
				params.push(match const_value {
					value::Value::Int(v) => {
						self.convert(value::to_llvm_var_type(&para_type), Value::Int(*v))
					}
					value::Value::Float(v) => {
						self.convert(value::to_llvm_var_type(&para_type), Value::Float(*v))
					}
					_ => {
						return Err(SysycError::LlvmSyntexError(format!(
							"Compile const value in call should not be {:?}",
							const_value
						)))
					}
				});
				continue;
			}
			if let Some(Attr::IRValue(v)) = param.get_attr(IRVALUE) {
				params
					.push(self.convert(value::to_llvm_var_type(&para_type), v.clone()));
			} else {
				return Err(SysycError::LlvmSyntexError(format!(
					"param of call {} has no value",
					val_decl.ident.clone()
				)));
			}
		}
		let var_type = match funcsymbol.var_type.0 {
			FuncRetType::Int => VarType::I32,
			FuncRetType::Float => VarType::F32,
			FuncRetType::Void => VarType::Void,
		};
		let target = self.funcemitter.as_mut().unwrap().visit_call_instr(
			var_type,
			val_decl.ident.clone(),
			params,
		);
		val_decl
			.set_attr(IRVALUE, Attr::IRValue(llvm::llvmop::Value::Temp(target)));
		Ok(())
	}
	fn visit_unary_expr(&mut self, val_decl: &mut UnaryExpr) -> Result<()> {
		val_decl.rhs.accept(self)?;
		// 检查是否有编译期常量
		if let Some(Attr::Value(const_value)) = val_decl.get_attr(VALUE) {
			let v = match const_value {
				value::Value::Int(v) => llvm::llvmop::Value::Int(*v),
				value::Value::Float(v) => llvm::llvmop::Value::Float(*v),
				_ => {
					return Err(SysycError::LlvmSyntexError(format!(
						"Compile const value in unary should not be {:?}",
						const_value
					)))
				}
			};
			val_decl.set_attr(IRVALUE, Attr::IRValue(v));
			return Ok(());
		}
		// 这里不检查rhs是否有编译期常量，因为如果是的话，UnaryExpr也一定是
		let expr_value = match val_decl.rhs.get_attr(IRVALUE) {
			Some(Attr::IRValue(v)) => v.clone(),
			_ => {
				return Err(SysycError::LlvmSyntexError(
					"unary expr has no value".to_string(),
				))
			}
		};
		let op = match val_decl.op {
			value::UnaryOp::Neg => {
				if expr_value.get_type() == llvm::llvmvar::VarType::F32 {
					Some(llvm::llvmop::ArithOp::Fsub)
				} else {
					Some(llvm::llvmop::ArithOp::Sub)
				}
			}
			value::UnaryOp::Not => Some(llvm::llvmop::ArithOp::Xor),
			// 不做运算
			value::UnaryOp::Plus => None,
		};
		if let Some(o) = op {
			let target = self.funcemitter.as_mut().unwrap().visit_arith_instr(
				Value::Int(0),
				o,
				expr_value,
			);
			val_decl
				.set_attr(IRVALUE, Attr::IRValue(llvm::llvmop::Value::Temp(target)));
		} else {
			val_decl.set_attr(IRVALUE, Attr::IRValue(expr_value));
		}
		Ok(())
	}
	fn visit_binary_expr(&mut self, val_decl: &mut BinaryExpr) -> Result<()> {
		val_decl.lhs.accept(self)?;
		val_decl.rhs.accept(self)?;
		// 编译期常量
		if let Some(Attr::Value(v)) = val_decl.get_attr(VALUE) {
			let v = match v {
				value::Value::Int(v) => llvm::llvmop::Value::Int(*v),
				value::Value::Float(v) => llvm::llvmop::Value::Float(*v),
				_ => {
					return Err(SysycError::LlvmSyntexError(format!(
						"Compile const value in binary should not be {:?}",
						v
					)))
				}
			};
			val_decl.set_attr(IRVALUE, Attr::IRValue(v.clone()));
			if let value::BinaryOp::Assign = val_decl.op {
				// lhs 如果有一个叫SYMBOL的attr，说明是一个变量，需要开一个新的Temp
				if let Some(Attr::VarSymbol(symbol)) = val_decl.lhs.get_attr(SYMBOL) {
					let temp = self
						.funcemitter
						.as_mut()
						.unwrap()
						.fresh_temp(value::to_llvm_var_type(&symbol.var_type));
					self
						.funcemitter
						.as_mut()
						.unwrap()
						.visit_assign_instr(temp.clone(), v);
					self
						.funcemitter
						.as_mut()
						.unwrap()
						.get_cur_basicblock()
						.symbol2temp
						.insert(symbol.id as usize, temp);
				} else if let Some(Attr::IRValue(Value::Temp(t))) =
					val_decl.lhs.get_attr(IRVALUE)
				{
					self.funcemitter.as_mut().unwrap().visit_assign_instr(t.clone(), v);
				} else {
					return Err(SysycError::LlvmSyntexError(
						"lhs of assign has no temp".to_string(),
					));
				}
			}
			return Ok(());
		}
		let lhs = match val_decl.lhs.get_attr(VALUE) {
			Some(Attr::Value(v)) => match v {
				value::Value::Int(v) => llvm::llvmop::Value::Int(*v),
				value::Value::Float(v) => llvm::llvmop::Value::Float(*v),
				_ => {
					return Err(SysycError::LlvmSyntexError(format!(
						"Compile const value in binary should not be {:?}",
						v
					)))
				}
			},
			_ => match val_decl.lhs.get_attr(IRVALUE) {
				Some(Attr::IRValue(v)) => v.clone(),
				_ => {
					return Err(SysycError::LlvmSyntexError(
						"lhs of binary expr has no value".to_string(),
					))
				}
			},
		};
		let rhs = match val_decl.rhs.get_attr(VALUE) {
			Some(Attr::Value(v)) => match v {
				value::Value::Int(v) => llvm::llvmop::Value::Int(*v),
				value::Value::Float(v) => llvm::llvmop::Value::Float(*v),
				_ => {
					return Err(SysycError::LlvmSyntexError(format!(
						"Compile const value in binary should not be {:?}",
						v
					)))
				}
			},
			_ => match val_decl.rhs.get_attr(IRVALUE) {
				Some(Attr::IRValue(v)) => v.clone(),
				_ => {
					return Err(SysycError::LlvmSyntexError(
						"lhs of binary expr has no value".to_string(),
					))
				}
			},
		};
		// TODO: 这里没有考虑void的情况，所以VarType为什么会包含Void啊
		let is_float = (lhs.get_type() == llvm::llvmvar::VarType::F32)
			|| (rhs.get_type() == llvm::llvmvar::VarType::F32);
		let op = match val_decl.op {
			value::BinaryOp::Add => {
				if is_float {
					Some(llvm::llvmop::ArithOp::Fadd)
				} else {
					Some(llvm::llvmop::ArithOp::Add)
				}
			}
			value::BinaryOp::Sub => {
				if is_float {
					Some(llvm::llvmop::ArithOp::Fsub)
				} else {
					Some(llvm::llvmop::ArithOp::Sub)
				}
			}
			value::BinaryOp::Mul => {
				if is_float {
					Some(llvm::llvmop::ArithOp::Fmul)
				} else {
					Some(llvm::llvmop::ArithOp::Mul)
				}
			}
			value::BinaryOp::Div => {
				if is_float {
					Some(llvm::llvmop::ArithOp::Fdiv)
				} else {
					Some(llvm::llvmop::ArithOp::Div)
				}
			}
			value::BinaryOp::Mod => Some(llvm::llvmop::ArithOp::Rem),
			value::BinaryOp::IDX => {
				let temp = self.funcemitter.as_mut().unwrap().visit_gep_instr(lhs, rhs);
				val_decl
					.set_attr(IRVALUE, Attr::IRValue(llvm::llvmop::Value::Temp(temp)));
				return Ok(()); // 这里直接返回，不需要再visit了
			}
			value::BinaryOp::Assign => {
				// lhs 如果有一个叫SYMBOL的attr，说明是一个变量，需要开一个新的Temp
				if let Some(Attr::VarSymbol(symbol)) = val_decl.lhs.get_attr(SYMBOL) {
					let temp = self
						.funcemitter
						.as_mut()
						.unwrap()
						.fresh_temp(value::to_llvm_var_type(&symbol.var_type));
					self
						.funcemitter
						.as_mut()
						.unwrap()
						.visit_assign_instr(temp.clone(), rhs);
					self
						.funcemitter
						.as_mut()
						.unwrap()
						.get_cur_basicblock()
						.symbol2temp
						.insert(symbol.id as usize, temp);
				} else if let Value::Temp(t) = lhs {
					self.funcemitter.as_mut().unwrap().visit_assign_instr(t.clone(), rhs);
				} else {
					return Err(SysycError::LlvmSyntexError(
						"lhs of assign has no temp".to_string(),
					));
				}
				return Ok(()); // 这里直接返回，不需要再visit了
			}
			_ => None,
		};
		if let Some(o) = op {
			let target =
				self.funcemitter.as_mut().unwrap().visit_arith_instr(lhs, o, rhs);
			val_decl
				.set_attr(IRVALUE, Attr::IRValue(llvm::llvmop::Value::Temp(target)));
		} else {
			let cmp_op = match val_decl.op {
				value::BinaryOp::LT => {
					if is_float {
						llvm::llvmop::CompOp::OLT
					} else {
						llvm::llvmop::CompOp::SLT
					}
				}
				value::BinaryOp::GT => {
					if is_float {
						llvm::llvmop::CompOp::OGT
					} else {
						llvm::llvmop::CompOp::SGT
					}
				}
				value::BinaryOp::GE => {
					if is_float {
						llvm::llvmop::CompOp::OGE
					} else {
						llvm::llvmop::CompOp::SGE
					}
				}
				value::BinaryOp::LE => {
					if is_float {
						llvm::llvmop::CompOp::OLE
					} else {
						llvm::llvmop::CompOp::SLE
					}
				}
				value::BinaryOp::EQ => {
					if is_float {
						llvm::llvmop::CompOp::OEQ
					} else {
						llvm::llvmop::CompOp::EQ
					}
				}
				value::BinaryOp::NE => {
					if is_float {
						llvm::llvmop::CompOp::ONE
					} else {
						llvm::llvmop::CompOp::NE
					}
				}
				_ => unreachable!(),
			};
			let target =
				self.funcemitter.as_mut().unwrap().visit_comp_instr(lhs, cmp_op, rhs);
			val_decl
				.set_attr(IRVALUE, Attr::IRValue(llvm::llvmop::Value::Temp(target)));
		}
		Ok(())
	}
	#[allow(unused_variables)]
	fn visit_break(&mut self, val_decl: &mut Break) -> Result<(), SysycError> {
		let target_bb_id = self.funcemitter.as_mut().unwrap().get_break_label();
		let target_label = self
			.funcemitter
			.as_mut()
			.unwrap()
			.get_basicblock(target_bb_id)
			.label
			.clone();

		self.funcemitter.as_mut().unwrap().visit_jump_instr(target_label);

		self.funcemitter.as_mut().unwrap().add_succ_to_cur_basicblock(target_bb_id);

		self.funcemitter.as_mut().unwrap().new_basicblock();
		Ok(())
	}
	#[allow(unused_variables)]
	fn visit_continue(&mut self, val_decl: &mut Continue) -> Result<()> {
		let target_bb_id = self.funcemitter.as_mut().unwrap().get_continue_label();
		let target_label = self
			.funcemitter
			.as_mut()
			.unwrap()
			.get_basicblock(target_bb_id)
			.label
			.clone();

		self.funcemitter.as_mut().unwrap().visit_jump_instr(target_label);

		self.funcemitter.as_mut().unwrap().add_succ_to_cur_basicblock(target_bb_id);

		self.funcemitter.as_mut().unwrap().new_basicblock();
		Ok(())
	}
	fn visit_return(&mut self, val_decl: &mut Return) -> Result<(), SysycError> {
		let ret_type = self.funcemitter.as_mut().unwrap().ret_type;
		if let Some(expr) = &mut val_decl.value {
			expr.accept(self)?;
			let value = if let Some(Attr::Value(v)) = expr.get_attr(VALUE) {
				match v {
					value::Value::Int(v) => self.convert(ret_type, Value::Int(*v)),
					value::Value::Float(v) => self.convert(ret_type, Value::Float(*v)),
					_ => {
						return Err(SysycError::LlvmSyntexError(format!(
							"Compile const value in return should not be {:?}",
							v
						)));
					}
				}
			} else {
				match expr.get_attr(IRVALUE) {
					Some(Attr::IRValue(v)) => self.convert(ret_type, v.clone()),
					_ => {
						return Err(SysycError::LlvmSyntexError(
							"return expr has no value".to_string(),
						))
					}
				}
			};
			self.funcemitter.as_mut().unwrap().visit_ret(Some(value));
		} else {
			self.funcemitter.as_mut().unwrap().visit_ret(None);
		}
		// exit basicblock的id是1
		self.funcemitter.as_mut().unwrap().add_succ_to_cur_basicblock(1);
		self.funcemitter.as_mut().unwrap().new_basicblock();
		Ok(())
	}
	fn visit_if(&mut self, val_decl: &mut If) -> Result<()> {
		val_decl.cond.accept(self)?;
		let cond_value = match val_decl.cond.get_attr(IRVALUE) {
			Some(Attr::IRValue(v)) => v.clone(),
			_ => {
				return Err(SysycError::LlvmSyntexError(
					"if cond has no value".to_string(),
				))
			}
		};
		let (beginlabel_id, beginlabel) =
			self.funcemitter.as_mut().unwrap().fresh_label();
		let (skiplabel_id, skiplabel) =
			self.funcemitter.as_mut().unwrap().fresh_label();
		let (exitlabel_id, exitlabel) =
			self.funcemitter.as_mut().unwrap().fresh_label();
		self.funcemitter.as_mut().unwrap().visit_jump_cond_instr(
			cond_value,
			beginlabel.clone(),
			skiplabel.clone(),
		);
		// 这里给CFG加了边，意味着symbol与temp的对应关系也随之流传过去
		self
			.funcemitter
			.as_mut()
			.unwrap()
			.add_succ_to_cur_basicblock(beginlabel_id);
		self.funcemitter.as_mut().unwrap().add_succ_to_cur_basicblock(skiplabel_id);
		// visitlabel时会切换basicblock
		self.funcemitter.as_mut().unwrap().visit_label(beginlabel_id);
		val_decl.body.accept(self)?;
		match val_decl.then {
			Some(ref mut then_block) => {
				self.funcemitter.as_mut().unwrap().visit_jump_instr(exitlabel.clone());

				self
					.funcemitter
					.as_mut()
					.unwrap()
					.add_succ_to_cur_basicblock(exitlabel_id);

				self.funcemitter.as_mut().unwrap().visit_label(skiplabel_id);
				then_block.accept(self)?;

				self.funcemitter.as_mut().unwrap().visit_jump_instr(exitlabel.clone());

				self
					.funcemitter
					.as_mut()
					.unwrap()
					.add_succ_to_cur_basicblock(exitlabel_id);

				self.funcemitter.as_mut().unwrap().visit_label(exitlabel_id);
				Ok(())
			}
			None => {
				self.funcemitter.as_mut().unwrap().visit_jump_instr(skiplabel);

				self
					.funcemitter
					.as_mut()
					.unwrap()
					.add_succ_to_cur_basicblock(skiplabel_id);

				self.funcemitter.as_mut().unwrap().visit_label(skiplabel_id);
				Ok(())
			}
		}
	}
	#[allow(unused_variables)]
	fn visit_while(&mut self, val_decl: &mut While) -> Result<(), SysycError> {
		let (beginlabel_id, beginlabel) =
			self.funcemitter.as_mut().unwrap().fresh_label();
		let (looplabel_id, looplabel) =
			self.funcemitter.as_mut().unwrap().fresh_label();
		let (breaklabel_id, breaklabel) =
			self.funcemitter.as_mut().unwrap().fresh_label();
		self.funcemitter.as_mut().unwrap().openloop(breaklabel_id, looplabel_id);

		self.funcemitter.as_mut().unwrap().visit_jump_instr(beginlabel.clone());

		self
			.funcemitter
			.as_mut()
			.unwrap()
			.add_succ_to_cur_basicblock(beginlabel_id);

		self.funcemitter.as_mut().unwrap().visit_label(beginlabel_id);
		val_decl.cond.accept(self)?;
		let cond_value = match val_decl.cond.get_attr(IRVALUE) {
			Some(Attr::IRValue(v)) => v.clone(),
			_ => {
				return Err(SysycError::LlvmSyntexError(
					"while cond has no value".to_string(),
				))
			}
		};
		let (beginlabel_for_jump_cond_instr_id, beginlabel_for_jump_cond_instr) =
			self.funcemitter.as_mut().unwrap().fresh_label();

		self.funcemitter.as_mut().unwrap().visit_jump_cond_instr(
			cond_value,
			beginlabel_for_jump_cond_instr.clone(),
			breaklabel.clone(),
		);

		self
			.funcemitter
			.as_mut()
			.unwrap()
			.add_succ_to_cur_basicblock(beginlabel_for_jump_cond_instr_id);

		self
			.funcemitter
			.as_mut()
			.unwrap()
			.add_succ_to_cur_basicblock(breaklabel_id);

		self
			.funcemitter
			.as_mut()
			.unwrap()
			.visit_label(beginlabel_for_jump_cond_instr_id);
		val_decl.body.accept(self)?;
		self.funcemitter.as_mut().unwrap().visit_jump_instr(looplabel);

		self.funcemitter.as_mut().unwrap().add_succ_to_cur_basicblock(looplabel_id);

		self.funcemitter.as_mut().unwrap().visit_label(looplabel_id);
		self.funcemitter.as_mut().unwrap().visit_jump_instr(beginlabel);

		self
			.funcemitter
			.as_mut()
			.unwrap()
			.add_succ_to_cur_basicblock(beginlabel_id);

		self.funcemitter.as_mut().unwrap().visit_label(breaklabel_id);
		Ok(())
	}

	#[allow(unused_variables)]
	fn visit_init_val_list(&mut self, val_decl: &mut InitValList) -> Result<()> {
		// 这里的attr来自visit_var_def
		let symbol = match val_decl.get_attr(CUR_SYMBOL) {
			Some(Attr::VarSymbol(s)) => s.clone(),
			_ => {
				return Err(SysycError::LlvmSyntexError(
					"init val has no symbol".to_string(),
				))
			}
		};
		for init_val in &mut val_decl.val_list {
			// 需要递归地告诉内部的InitValList，这个InitValList是属于哪个数组的
			init_val.set_attr(CUR_SYMBOL, Attr::VarSymbol(symbol.clone()));
			init_val.accept(self)?;
			// 检查 init_val 是否是常量
			if let Some(Attr::Value(v)) = init_val.get_attr(VALUE) {
				let addr = match init_val.get_attr(INDEX) {
					Some(Attr::InitListPosition(index)) => {
						llvm::llvmop::Value::Int(*index as i32)
					}
					_ => {
						return Err(SysycError::LlvmSyntexError(
							"init val has no index".to_string(),
						))
					}
				};
				let symbol_temp =
					self.funcemitter.as_mut().unwrap().get_cur_basicblock().symbol2temp
						[&(symbol.id as usize)]
						.clone();
				let temp = self
					.funcemitter
					.as_mut()
					.unwrap()
					.visit_gep_instr(Value::Temp(symbol_temp), addr);
				let llvm_value = match v {
					value::Value::Int(v) => Value::Int(*v),
					value::Value::Float(v) => Value::Float(*v),
					_ => {
						return Err(SysycError::LlvmSyntexError(format!(
							"Compile const value in init val should not be {:?}",
							v
						)))
					}
				};
				self
					.funcemitter
					.as_mut()
					.unwrap()
					.visit_store_instr(llvm_value, Value::Temp(temp));
			} else if let Some(Attr::IRValue(v)) = init_val.get_attr(IRVALUE) {
				let addr = match init_val.get_attr(INDEX) {
					Some(Attr::InitListPosition(index)) => {
						llvm::llvmop::Value::Int(*index as i32)
					}
					_ => {
						return Err(SysycError::LlvmSyntexError(
							"init val has no index".to_string(),
						))
					}
				};
				let symbol_temp =
					self.funcemitter.as_mut().unwrap().get_cur_basicblock().symbol2temp
						[&(symbol.id as usize)]
						.clone();
				let temp = self
					.funcemitter
					.as_mut()
					.unwrap()
					.visit_gep_instr(Value::Temp(symbol_temp), addr);
				self
					.funcemitter
					.as_mut()
					.unwrap()
					.visit_store_instr(v.clone(), Value::Temp(temp));
			}
		}
		Ok(())
	}
	fn visit_literal_float(&mut self, val_decl: &mut LiteralFloat) -> Result<()> {
		val_decl.set_attr(IRVALUE, Attr::IRValue(Value::Float(val_decl.value)));
		Ok(())
	}
	fn visit_literal_int(&mut self, val_decl: &mut LiteralInt) -> Result<()> {
		val_decl.set_attr(IRVALUE, Attr::IRValue(Value::Int(val_decl.value)));
		Ok(())
	}
	fn visit_variable(&mut self, node: &mut Variable) -> Result<()> {
		if let Attr::VarSymbol(symbol) = node.get_attr(SYMBOL).ok_or_else(|| {
			SysycError::LlvmSyntexError(format!(
				"var {} has no symbol",
				node.ident.clone()
			))
		})? {
			if !symbol.is_global {
				node.set_attr(
					IRVALUE,
					Attr::IRValue(Value::Temp(
						self.funcemitter.as_mut().unwrap().get_cur_basicblock().symbol2temp
							[&(symbol.id as usize)]
							.clone(),
					)),
				);
			} else {
				node.set_attr(
					IRVALUE,
					Attr::IRValue(Value::Temp(Temp::new_global(
						symbol.ident.clone(),
						value::to_llvm_var_type(&symbol.var_type),
					))),
				);
			}
		}
		Ok(())
	}
}
