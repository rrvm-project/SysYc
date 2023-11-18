use std::collections::HashMap;

use ast::{tree::*, visitor::Visitor};
use attr::{Attr, Attrs};
use llvm::{
	func::LlvmFunc,
	llvmfuncemitter::LlvmFuncEmitter,
	llvmop::{ConvertOp, Value},
	llvmvar::VarType,
	temp::Temp,
	LlvmProgram, basicblock::BasicBlock, cfg::CFG,
};
use rrvm_symbol::VarSymbol;
// use namer::{
// 	namer::{COMPILE_CONST, COMPILE_CONST_INDEX, INDEX, SYMBOL_NUMBER},
// 	utils::DataFromNamer,
// };
use utils::{errors::Result, Label, SysycError};
use value::{FuncRetType, BType};

static VALUE: &str = "value";

// 意思是 irgen 过程中会给节点挂上的 attribute，内容是llvm::llvmop::Value, 名字可能名不副实
static IRVALUE: &str = "irvalue";
static FUNC_SYMBOL: &str = "func_symbol";
static SYMBOL: &str = "symbol";

// 为了实现 SSA，我将每一个变量都存入栈中，与变量绑定的 Temp 存储了地址，也就是在栈中的位置
// 每次要使用一个变量的值，都需要一条 load 指令
// 每次为这个变量赋值，都需要一条store指令
// 在 visit_LVal 中，会给LVal节点同时挂上VALUE和ADDRESS的attr，而在其余visit方法中，不会挂ADDRESS，只会在需要的时候挂VALUE
static ADDRESS: &str = "address";
pub struct LlvmIrGen {
	pub funcemitter: Option<LlvmFuncEmitter>,
	pub funcs: Vec<LlvmFunc>,
	// BasicBlock.id -> Temp
	pub symbol2temp: HashMap<usize, HashMap<VarSymbol, Temp>>,
	// pub data: DataFromNamer,
}

impl LlvmIrGen {
	pub fn transform(&mut self, mut program: Program) -> Result<()> {
		program.accept(self)
	}
	pub fn emit_program(self) -> LlvmProgram {
		LlvmProgram {
			funcs: self.funcs,
			global_vars: HashMap::new(),
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
}

impl Visitor for LlvmIrGen {
	fn visit_program(&mut self, program: &mut Program) -> Result<()> {
		for comp_unit in &mut program.comp_units {
			comp_unit.accept(self)?;
		}
		Ok(())
	}
	fn visit_func_decl(
		&mut self,
		val_decl: &mut FuncDecl,
	) -> Result<()> {
		let ret_type = match val_decl.ret_type {
			FuncRetType::Int => llvm::llvmvar::VarType::I32,
			FuncRetType::Float => llvm::llvmvar::VarType::F32,
			FuncRetType::Void => llvm::llvmvar::VarType::Void,
		};

		let entry = BasicBlock::new(0, Label::new("entry"), vec![]);

		self.funcemitter = Some(LlvmFuncEmitter::new(
			val_decl.ident.clone(),
			ret_type,
			vec![],
			entry
		));

		for param in &mut val_decl.formal_params {
			param.accept(self)?;
		}

		val_decl.block.accept(self)?;
		self.funcs.push(self.funcemitter.take().unwrap().visit_end());
		self.funcemitter = None;
		Ok(())
	}
	fn visit_var_decl(
		&mut self,
		val_decl: &mut VarDecl,
	) -> Result<()> {
		for def in val_decl.defs.iter_mut() {
			def.accept(self)?;
		}
		Ok(())
	}
	fn visit_var_def(&mut self, val_decl: &mut VarDef) -> Result<()> {
		let symbol: VarSymbol = val_decl.get_attr(SYMBOL).ok_or_else(|| SysycError::LlvmSyntexError(format!(
			"var def {} has no symbol",
			val_decl.ident.clone())))?.into();
		
		let var_type = symbol.var_type;
		// 分配空间与初始化
		if var_type.2.len() > 0 {
			// 是数组
			let tp = match var_type.1 {
				BType::Int => llvm::llvmvar::VarType::I32Ptr,
				BType::Float => llvm::llvmvar::VarType::F32Ptr,
			};
			// TODO: 这里的Value里面应该是个usize还是个i32呢
			let mut size = 4;
			for i in var_type.2{
				size *= i as i32;
			}
			let temp = self
				.funcemitter
				.as_mut()
				.unwrap()
				.visit_alloc_instr(tp, Value::Int(size));
			self.symbol2temp[&self.funcemitter.as_mut().unwrap().get_cur_basicblock().id].insert(symbol.clone(), temp);
			// 初始化
			if let Some(init) = &mut val_decl.init {
				// 这里应当是一个初始化列表，设置一个Attr告知正在对哪个数组做初始化
				init.set_attr(SYMBOL, Attr::VarSymbol(symbol.clone()));
				init.accept(self)?;
			}
		} else {
			// 是标量
			let tp = match var_type.1 {
				BType::Int => llvm::llvmvar::VarType::I32,
				BType::Float => llvm::llvmvar::VarType::F32,
			};
			let temp = self
				.funcemitter
				.as_mut()
				.unwrap()
				.fresh_temp(tp.clone());
			self.symbol2temp[&self.funcemitter.as_mut().unwrap().get_cur_basicblock().id].insert(symbol.clone(), temp.clone());
			// 初始化
			// TODO: 这里应该从val_decl.init中获取值呢，还是从symbol.const_or_global_initial_value中获取值呢
			if let Some(init) = &mut val_decl.init {
				init.accept(self)?;
				if let Some(Attr::Value(const_value)) =
					init.get_attr(VALUE)
				{
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
	fn visit_formal_param(
		&mut self,
		val_decl: &mut FormalParam,
	) -> Result<()> {
		let var_type = match (val_decl.type_t, val_decl.dim_list.len()) {
			(BType::Int, 0) => llvm::llvmvar::VarType::I32,
			(BType::Float, 0) => llvm::llvmvar::VarType::F32,
			(BType::Int, _) => llvm::llvmvar::VarType::I32Ptr,
			(BType::Float, _) => llvm::llvmvar::VarType::F32Ptr,
		};
		let tmp = self.funcemitter.as_mut().unwrap().visit_formal_param(var_type);
		match val_decl.get_attr(SYMBOL) {
			Some(Attr::VarSymbol(symbol)) => self.symbol2temp[&self.funcemitter.as_mut().unwrap().get_cur_basicblock().id].insert(symbol.clone(), tmp)
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
	fn visit_func_call(
		&mut self,
		val_decl: &mut FuncCall,
	) -> Result<()> {
		let funcsymbol = match val_decl.get_attr(FUNC_SYMBOL) {
			Some(Attr::FuncSymbol(symbol)) => symbol,
			_ => {
				return Err(SysycError::LlvmSyntexError(format!(
					"call {} has no funcsymbol",
					val_decl.ident.clone()
				)))
			}
		};
		let mut params = vec![];
		for (param, para_type) in val_decl.params.iter_mut().zip(funcsymbol.var_type.1)
		{
			param.accept(self)?;
			if let Some(Attr::Value(const_value)) = param.get_attr(VALUE) {
				params.push(match const_value {
					value::Value::Int(v) => {
						self.convert(value::to_llvmVarType(&para_type), Value::Int(*v))
					}
					value::Value::Float(v) => {
						self.convert(value::to_llvmVarType(&para_type), Value::Float(*v))
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
				params.push(self.convert(value::to_llvmVarType(&para_type), v.clone()));
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
		val_decl.set_attr(IRVALUE, Attr::IRValue(llvm::llvmop::Value::Temp(target)));
		Ok(())
	}
	fn visit_unary_expr(
		&mut self,
		val_decl: &mut UnaryExpr,
	) -> Result<()> {
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
			val_decl.set_attr(IRVALUE, Attr::IRValue(llvm::llvmop::Value::Temp(target)));
		} else {
			val_decl.set_attr(IRVALUE, Attr::IRValue(expr_value));
		}
		Ok(())
	}
	fn visit_binary_expr(
		&mut self,
		val_decl: &mut BinaryExpr,
	) -> Result<()> {
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
			val_decl.set_attr(IRVALUE, Attr::IRValue(v));
			if let value::BinaryOp::Assign = val_decl.op {
				// 不能使用 lhs attrs中的VALUE，应当使用ADDRESS
				let addr = match val_decl.lhs.get_attr(ADDRESS) {
					Some(Attr::Value(v)) => v.clone(),
					_ => {
						return Err(SysycError::LlvmSyntexError(
							"lhs of assign has no address".to_string(),
						))
					}
				};
				let rhs = match val_decl.rhs.get_attr(COMPILE_CONST) {
					Some(Attr::CompileConstValue(v)) => match v {
						CompileConstValue::Int(v) => llvm::llvmop::Value::Int(*v),
						CompileConstValue::Float(v) => llvm::llvmop::Value::Float(*v),
						_ => {
							return Err(SysycError::LlvmSyntexError(format!(
								"Compile const value in binary should not be {:?}",
								v
							)))
						}
					},
					_ => {
						return Err(SysycError::LlvmSyntexError(
							"rhs of assign has no value".to_string(),
						))
					}
				};
				self.funcemitter.as_mut().unwrap().visit_store_instr(rhs, addr);
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
			value::BinaryOp::Assign => {
				let addr = match val_decl.lhs.get_attr(ADDRESS) {
					Some(Attr::Value(v)) => v.clone(),
					_ => {
						return Err(SysycError::LlvmSyntexError(
							"lhs of assign has no address".to_string(),
						))
					}
				};
				self.funcemitter.as_mut().unwrap().visit_store_instr(rhs, addr);
				return Ok(()); // 这里直接返回，不需要再visit了
			}
			_ => None,
		};
		if let Some(o) = op {
			let target =
				self.funcemitter.as_mut().unwrap().visit_arith_instr(lhs, o, rhs);
			val_decl.set_attr(IRVALUE, Attr::IRValue(llvm::llvmop::Value::Temp(target)));
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
			val_decl.set_attr(IRVALUE, Attr::IRValue(llvm::llvmop::Value::Temp(target)));
		}
		Ok(())
	}
	#[allow(unused_variables)]
	fn visit_break(&mut self, val_decl: &mut Break) -> Result<(), SysycError> {
		let label = self.funcemitter.as_ref().unwrap().get_break_label();
		self.funcemitter.as_mut().unwrap().visit_jump_instr(label);
		Ok(())
	}
	#[allow(unused_variables)]
	fn visit_continue(
		&mut self,
		val_decl: &mut Continue,
	) -> Result<()> {
		let label = self.funcemitter.as_ref().unwrap().get_continue_label();
		self.funcemitter.as_mut().unwrap().visit_jump_instr(label);
		Ok(())
	}
	fn visit_return(&mut self, val_decl: &mut Return) -> Result<(), SysycError> {
		let ret_type = self.funcemitter.as_mut().unwrap().ret_type;
		if let Some(expr) = &mut val_decl.value {
			expr.accept(self)?;
			let value = if let Some(Attr::CompileConstValue(v)) =
				expr.get_attr(COMPILE_CONST)
			{
				match v {
					CompileConstValue::Int(v) => self.convert(ret_type, Value::Int(*v)),
					CompileConstValue::Float(v) => {
						self.convert(ret_type, Value::Float(*v))
					}
					_ => {
						return Err(SysycError::LlvmSyntexError(format!(
							"Compile const value in return should not be {:?}",
							v
						)));
					}
				}
			} else {
				match expr.get_attr(VALUE) {
					Some(Attr::Value(v)) => self.convert(ret_type, v.clone()),
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
		Ok(())
	}
	fn visit_if(&mut self, val_decl: &mut If) -> Result<()> {
		val_decl.cond.accept(self)?;
		let cond_value = match val_decl.cond.get_attr(VALUE) {
			Some(Attr::Value(v)) => v.clone(),
			_ => {
				return Err(SysycError::LlvmSyntexError(
					"if cond has no value".to_string(),
				))
			}
		};
		let beginlabel = self.funcemitter.as_mut().unwrap().fresh_label();
		let skiplabel = self.funcemitter.as_mut().unwrap().fresh_label();
		let exitlabel = self.funcemitter.as_mut().unwrap().fresh_label();
		self.funcemitter.as_mut().unwrap().visit_jump_cond_instr(
			cond_value,
			beginlabel.clone(),
			skiplabel.clone(),
		);
		self.funcemitter.as_mut().unwrap().visit_label(beginlabel);
		val_decl.body.accept(self)?;
		match val_decl.then {
			Some(ref mut then_block) => {
				self.funcemitter.as_mut().unwrap().visit_jump_instr(exitlabel.clone());
				self.funcemitter.as_mut().unwrap().visit_label(skiplabel);
				then_block.accept(self)?;
				self.funcemitter.as_mut().unwrap().visit_label(exitlabel);
				Ok(())
			}
			None => {
				self.funcemitter.as_mut().unwrap().visit_label(skiplabel);
				Ok(())
			}
		}
	}
	fn visit_while(&mut self, val_decl: &mut While) -> Result<(), SysycError> {
		let beginlabel = self.funcemitter.as_mut().unwrap().fresh_label();
		let looplabel = self.funcemitter.as_mut().unwrap().fresh_label();
		let breaklabel = self.funcemitter.as_mut().unwrap().fresh_label();
		self
			.funcemitter
			.as_mut()
			.unwrap()
			.openloop(breaklabel.clone(), looplabel.clone());
		self.funcemitter.as_mut().unwrap().visit_label(beginlabel.clone());
		val_decl.cond.accept(self)?;
		let cond_value = match val_decl.cond.get_attr(VALUE) {
			Some(Attr::Value(v)) => v.clone(),
			_ => {
				return Err(SysycError::LlvmSyntexError(
					"while cond has no value".to_string(),
				))
			}
		};
		let beginlabel_for_jump_cond_instr =
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
			.visit_label(beginlabel_for_jump_cond_instr);
		val_decl.body.accept(self)?;
		self.funcemitter.as_mut().unwrap().visit_label(looplabel);
		self.funcemitter.as_mut().unwrap().visit_jump_instr(beginlabel);
		self.funcemitter.as_mut().unwrap().visit_label(breaklabel);
		Ok(())
	}
	fn visit_init_val_list(
		&mut self,
		val_decl: &mut InitValList,
	) -> Result<()> {
		// 这里的attr来自visit_var_def
		let symbol_id = match val_decl.get_attr(SYMBOL_NUMBER) {
			Some(Attr::VarSymbol(id)) => *id,
			_ => {
				return Err(SysycError::LlvmSyntexError(
					"init val has no symbol".to_string(),
				))
			}
		};
		let symbol = self.data.var_symbols[symbol_id].clone();
		for init_val in &mut val_decl.val_list {
			// 需要递归地告诉内部的InitValList，这个InitValList是属于哪个数组的
			init_val.set_attr(SYMBOL_NUMBER, Attr::VarSymbol(symbol_id));
			init_val.accept(self)?;
			match init_val.get_attr(VALUE) {
				// 有 Some 说明这个init_val不是一个InitValList
				Some(Attr::Value(v)) => {
					let addr = match init_val.get_attr(INDEX) {
						Some(Attr::UIntValue(index)) => {
							llvm::llvmop::Value::Int(*index as i32)
						}
						_ => {
							return Err(SysycError::LlvmSyntexError(
								"init val has no index".to_string(),
							))
						}
					};
					let temp = self.funcemitter.as_mut().unwrap().visit_gep_instr(
						Value::Temp(symbol.temp.as_ref().unwrap().clone()),
						addr,
					);
					self
						.funcemitter
						.as_mut()
						.unwrap()
						.visit_store_instr(v.clone(), Value::Temp(temp));
				}
				// None 说明这个init_val是一个InitValList, 会被递归地调用，这里什么都不用做
				None => {}
				_ => {
					return Err(SysycError::LlvmSyntexError(
						"init val has no value".to_string(),
					))
				}
			};
		}
		Ok(())
	}
	fn visit_literal_float(
		&mut self,
		val_decl: &mut LiteralFloat,
	) -> Result<()> {
		let bb: BasicBlock = self.funcemitter.as_mut().unwrap().cur_bb.clone();
		val_decl.set_attr(
			VALUE,
			Attr::Value(llvm::llvmop::Value::Float(val_decl.value)),
		);
		Ok(())
	}
	fn visit_literal_int(
		&mut self,
		val_decl: &mut LiteralInt,
	) -> Result<()> {
		val_decl
			.set_attr(VALUE, Attr::Value(llvm::llvmop::Value::Int(val_decl.value)));
		Ok(())
	}
    fn visit_variable(&mut self, node: &mut Variable) -> Result<()> {
        todo!()
    }
}