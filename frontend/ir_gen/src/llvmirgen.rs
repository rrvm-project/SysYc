use ast::{tree::*, visitor::Visitor};
use attr::{Attr, Attrs, CompileConstValue};
use llvm::llvmop::Value;
use llvm::llvmvar::VarType;
use llvm::temp::Temp;
use llvm::{func::LlvmFunc, llvmfuncemitter::LlvmFuncEmitter, LlvmProgram};
use namer::namer::{SYMBOL_NUMBER, TYPE, COMPILE_CONST, COMPILE_CONST_INDEX, INDEX};
use namer::utils::DataFromNamer;
use utils::SysycError;

static VALUE: &str = "value";

// 为了实现 SSA，我将每一个变量都存入栈中，与变量绑定的 Temp 存储了地址，也就是在栈中的位置
// 每次要使用一个变量的值，都需要一条 load 指令
// 每次为这个变量赋值，都需要一条store指令
// 在 visit_LVal 中，会给LVal节点同时挂上VALUE和ADDRESS的attr，而在其余visit方法中，不会挂ADDRESS，只会在需要的时候挂VALUE
static ADDRESS: &str = "address";
pub struct LlvmIrGen {
	pub funcemitter: Option<LlvmFuncEmitter>,
	pub funcs: Vec<LlvmFunc>,
	pub data: DataFromNamer,
}

impl LlvmIrGen {
	pub fn transform(&mut self, mut program: Program) -> Result<(), SysycError> {
		program.accept(self)?;
		Ok(())
	}
	pub fn emit_program(self) -> LlvmProgram {
		LlvmProgram {
			funcs: self.funcs,
			// funcs: vec![self.funcemitter.emit_func()],
			global_vars: vec![],
		}
	}
}

impl Visitor for LlvmIrGen {
	fn visit_program(&mut self, program: &mut Program) -> Result<(), SysycError> {
		// TODO: 这个 for 循环如果改成迭代器访问的话，不知道如何传出错误
		for comp_unit in &mut program.comp_units {
			comp_unit.accept(self)?;
		}
		Ok(())
	}
	fn visit_func_decl(
		&mut self,
		val_decl: &mut FuncDecl,
	) -> Result<(), SysycError> {
		let ret_type = match val_decl.func_type {
			ast::FuncType::Int => llvm::llvmvar::VarType::I32,
			ast::FuncType::Float => llvm::llvmvar::VarType::F32,
			ast::FuncType::Void => llvm::llvmvar::VarType::Void,
		};
		self.funcemitter = Some(LlvmFuncEmitter::new(
			val_decl.ident.clone(),
			ret_type,
			vec![],
		));
		for param in &mut val_decl.formal_params {
			param.accept(self)?;
		}
		val_decl.block.accept(self)?;
		Ok(())
	}
	fn visit_var_decl(
		&mut self,
		val_decl: &mut VarDecl,
	) -> Result<(), SysycError> {
		for def in val_decl.defs.iter_mut() {
            def.accept(self)?;
        }
        Ok(())
	}
	fn visit_var_def(&mut self, val_decl: &mut VarDef) -> Result<(), SysycError> {
		let symbol = match val_decl.get_attr(SYMBOL_NUMBER) {
			Some(Attr::UIntValue(id)) => &mut self.data.var_symbols[*id],
			_ => {
				return Err(SysycError::LlvmSyntexError(format!(
					"var def {} has no symbol",
					val_decl.ident.clone()
				)))
			}
		};
		// 分配空间与初始化
		if let Some(_) = &mut val_decl.dim_list {
			// 是数组
			if let Some(Attr::Type(var_type)) = val_decl.get_attr(TYPE) {
				let tp = match var_type.base_type {
					ir_type::builtin_type::BaseType::Int => {
						llvm::llvmvar::VarType::I32Ptr
					}
					ir_type::builtin_type::BaseType::Float => {
						llvm::llvmvar::VarType::F32Ptr
					}
					ir_type::builtin_type::BaseType::Void => {
						return Err(SysycError::LlvmSyntexError(format!(
							"var def {} has void type",
							val_decl.ident.clone()
						)))
					}
				};
				// TODO: 这里的Value里面应该是个usize还是个i32呢
				let temp = self.funcemitter.as_mut().unwrap().visit_alloc_instr(tp, Value::Int(var_type.size() as i32));
				symbol.temp = Some(temp);
			} else {
				return Err(SysycError::LlvmSyntexError(format!(
					"var def {} has no type",
					val_decl.ident.clone()
				)));				
			}
			// 初始化
			if let Some(init) = &mut val_decl.init {
				// 这里应当是一个初始化列表，设置一个Attr告知正在对哪个数组做初始化
				init.set_attr(SYMBOL_NUMBER, Attr::UIntValue(symbol.id));
				init.accept(self)?;
			}
		} else {
			// TODO: 是常量，这里也选择分配到栈上，为了保证SSA
			if let Some(Attr::Type(var_type)) = val_decl.get_attr(TYPE) {
				let tp = match var_type.base_type {
					ir_type::builtin_type::BaseType::Int => {
						llvm::llvmvar::VarType::I32
					}
					ir_type::builtin_type::BaseType::Float => {
						llvm::llvmvar::VarType::F32
					}
					ir_type::builtin_type::BaseType::Void => {
						return Err(SysycError::LlvmSyntexError(format!(
							"var def {} has void type",
							val_decl.ident.clone()
						)))
					}
				};
				let temp = self.funcemitter.as_mut().unwrap().visit_alloc_instr(tp, Value::Int(var_type.size() as i32));
				symbol.temp = Some(temp);
			} else {
				return Err(SysycError::LlvmSyntexError(format!(
					"var def {} has no type",
					val_decl.ident.clone()
				)));				
			}
			// 初始化
			// TODO: 这里应该从val_decl.init中获取值呢，还是从symbol.const_or_global_initial_value中获取值呢
			if let Some(init) = &mut val_decl.init{
				let temp = symbol.temp.as_ref().unwrap().clone();
				if let Some(Attr::CompileConstValue(const_value)) = init.get_attr(COMPILE_CONST){
					match const_value {
						CompileConstValue::Int(v) => {
							self.funcemitter.as_mut().unwrap().visit_store_instr(Value::Int(*v), Value::Temp(temp));
						},
						CompileConstValue::Float(v) => {
							self.funcemitter.as_mut().unwrap().visit_store_instr(Value::Float(*v), Value::Temp(temp));
						},
						_ => {
							return Err(SysycError::LlvmSyntexError(format!(
								"const value for {} should not be other than float and int",
								val_decl.ident.clone()
							)))
						}
					}
				} else {
					init.accept(self)?;
					let init_value = match init.get_attr(VALUE) {
						Some(Attr::Value(v)) => v.clone(),
						_ => {
							return Err(SysycError::LlvmSyntexError(format!(
								"init value for {} has no value",
								val_decl.ident.clone()
							)))
						}
					};
					self.funcemitter.as_mut().unwrap().visit_store_instr(init_value, Value::Temp(temp));
				}
			}
		}
        Ok(())
	}
	fn visit_formal_param(
		&mut self,
		val_decl: &mut FormalParam,
	) -> Result<(), SysycError> {
		let var_type = match val_decl.type_t {
			ast::VarType::Int => {
				if val_decl.dim_list.is_none() {
					llvm::llvmvar::VarType::I32
				} else {
					llvm::llvmvar::VarType::I32Ptr
				}
			}
			ast::VarType::Float => {
				if val_decl.dim_list.is_none() {
					llvm::llvmvar::VarType::F32
				} else {
					llvm::llvmvar::VarType::F32Ptr
				}
			}
		};
		self.funcemitter.as_mut().unwrap().visit_formal_param(var_type);
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
	) -> Result<(), SysycError> {
		let mut params = vec![];
		for param in &mut val_decl.params {
			param.accept(self)?;
			if let Some(Attr::Value(v)) = param.get_attr(VALUE) {
				params.push(v.clone());
			} else {
				return Err(SysycError::LlvmSyntexError(format!(
					"param of call {} has no value",
					val_decl.ident.clone()
				)));
			}
		}
		let funcsymbol_id = match val_decl.get_attr(SYMBOL_NUMBER) {
			Some(Attr::FuncSymbol(id)) => *id,
			_ => {
				return Err(SysycError::LlvmSyntexError(format!(
					"call {} has no funcsymbol",
					val_decl.ident.clone()
				)))
			}
		};
		let funcsymbol = &self.data.func_symbols[funcsymbol_id];
		let var_type = match funcsymbol.ret_t.base_type {
			ir_type::builtin_type::BaseType::Int => {
				if funcsymbol.ret_t.dims.len() == 0 {
					VarType::I32
				} else {
					VarType::I32Ptr
				}
			}
			ir_type::builtin_type::BaseType::Float => {
				if funcsymbol.ret_t.dims.len() == 0 {
					VarType::F32
				} else {
					VarType::F32Ptr
				}
			}
			ir_type::builtin_type::BaseType::Void => VarType::Void,
		};
		let target = self.funcemitter.as_mut().unwrap().visit_call_instr(
			var_type,
			val_decl.ident.clone(),
			params,
		);
		val_decl.set_attr(VALUE, Attr::Value(llvm::llvmop::Value::Temp(target)));
		Ok(())
	}
	fn visit_unary_expr(
		&mut self,
		val_decl: &mut UnaryExpr,
	) -> Result<(), SysycError> {
		val_decl.rhs.accept(self)?;
		let expr_value = match val_decl.rhs.get_attr(VALUE) {
			Some(Attr::Value(v)) => v.clone(),
			_ => {
				return Err(SysycError::LlvmSyntexError(format!(
					"unary expr has no value"
				)))
			}
		};
		let op = match val_decl.op {
			ast::UnaryOp::Neg => {
				if expr_value.get_type() == llvm::llvmvar::VarType::F32 {
					Some(llvm::llvmop::ArithOp::Fsub)
				} else {
					Some(llvm::llvmop::ArithOp::Sub)
				}
			}
			ast::UnaryOp::Not => Some(llvm::llvmop::ArithOp::Xor),
			// 不做运算
			ast::UnaryOp::Plus => None
		};
		if op.is_some() {
			let target = self.funcemitter.as_mut().unwrap().visit_arith_instr(
				Value::Int(0),
				op.unwrap(),
				expr_value,
			);
			val_decl.set_attr(VALUE, Attr::Value(llvm::llvmop::Value::Temp(target)));
		} else {
			val_decl.set_attr(VALUE, Attr::Value(expr_value));
		}
		Ok(())
	}
	fn visit_binary_expr(
		&mut self,
		val_decl: &mut BinaryExpr,
	) -> Result<(), SysycError> {
		val_decl.lhs.accept(self)?;
		val_decl.rhs.accept(self)?;
		let lhs = match val_decl.lhs.get_attr(VALUE) {
			Some(Attr::Value(v)) => v.clone(),
			_ => {
				return Err(SysycError::LlvmSyntexError(format!(
					"lhs of binary expr has no value"
				)))
			}
		};
		let rhs = match val_decl.rhs.get_attr(VALUE) {
			Some(Attr::Value(v)) => v.clone(),
			_ => {
				return Err(SysycError::LlvmSyntexError(format!(
					"rhs of binary expr has no value"
				)))
			}
		};
		// TODO: 这里没有考虑void的情况，所以VarType为什么会包含Void啊
		let is_float = lhs.get_type() == llvm::llvmvar::VarType::F32
			|| rhs.get_type() == llvm::llvmvar::VarType::F32;
		let op = match val_decl.op {
			ast::BinaryOp::Add => {
				if is_float {
					Some(llvm::llvmop::ArithOp::Fadd)
				} else {
					Some(llvm::llvmop::ArithOp::Add)
				}
			}
			ast::BinaryOp::Sub => {
				if is_float {
					Some(llvm::llvmop::ArithOp::Fsub)
				} else {
					Some(llvm::llvmop::ArithOp::Sub)
				}
			}
			ast::BinaryOp::Mul => {
				if is_float {
					Some(llvm::llvmop::ArithOp::Fmul)
				} else {
					Some(llvm::llvmop::ArithOp::Mul)
				}
			}
			ast::BinaryOp::Div => {
				if is_float {
					Some(llvm::llvmop::ArithOp::Fdiv)
				} else {
					Some(llvm::llvmop::ArithOp::Div)
				}
			}
			ast::BinaryOp::Mod => {
				if is_float {
					Some(llvm::llvmop::ArithOp::Frem)
				} else {
					Some(llvm::llvmop::ArithOp::Rem)
				}
			}
			ast::BinaryOp::Assign => {
				// 不能使用 lhs attrs中的VALUE，应当使用ADDRESS
				let addr = match val_decl.lhs.get_attr(ADDRESS) {
					Some(Attr::Value(v)) => v.clone(),
					_ => {
						return Err(SysycError::LlvmSyntexError(format!(
							"lhs of assign has no address"
						)))
					}
				};
				self.funcemitter.as_mut().unwrap().visit_store_instr(rhs.clone(), addr);
				return Ok(()); // 这里直接返回，不需要再visit了
			},
			_ => None,
		};
		if op.is_some() {
			let target = self.funcemitter.as_mut().unwrap().visit_arith_instr(
				lhs,
				op.unwrap(),
				rhs,
			);
			val_decl.set_attr(VALUE, Attr::Value(llvm::llvmop::Value::Temp(target)));
		} else {
			let cmp_op = match val_decl.op {
				ast::BinaryOp::LT => {
					if is_float {
						llvm::llvmop::CompOp::OLT
					} else {
						llvm::llvmop::CompOp::SLT
					}
				}
				ast::BinaryOp::GT => {
					if is_float {
						llvm::llvmop::CompOp::OGT
					} else {
						llvm::llvmop::CompOp::SGT
					}
				}
				ast::BinaryOp::GE => {
					if is_float {
						llvm::llvmop::CompOp::OGE
					} else {
						llvm::llvmop::CompOp::SGE
					}
				}
				ast::BinaryOp::LE => {
					if is_float {
						llvm::llvmop::CompOp::OLE
					} else {
						llvm::llvmop::CompOp::SLE
					}
				}
				ast::BinaryOp::EQ => {
					if is_float {
						llvm::llvmop::CompOp::OEQ
					} else {
						llvm::llvmop::CompOp::EQ
					}
				}
				ast::BinaryOp::NE => {
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
			val_decl.set_attr(VALUE, Attr::Value(llvm::llvmop::Value::Temp(target)));
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
	) -> Result<(), SysycError> {
		let label = self.funcemitter.as_ref().unwrap().get_continue_label();
		self.funcemitter.as_mut().unwrap().visit_jump_instr(label);
        Ok(())
	}
	fn visit_return(&mut self, val_decl: &mut Return) -> Result<(), SysycError> {
		if let Some(expr) = &mut val_decl.value {
			expr.accept(self)?;
			let value = match expr.get_attr(VALUE) {
				Some(Attr::Value(v)) => v.clone(),
				_ => {
					return Err(SysycError::LlvmSyntexError(format!(
						"return expr has no value"
					)))
				}
			};
			self.funcemitter.as_mut().unwrap().visit_ret(value);
		} else {
			self.funcemitter.as_mut().unwrap().visit_ret(llvm::llvmop::Value::Void);
		}
		Ok(())
	}
	fn visit_if(&mut self, val_decl: &mut If) -> Result<(), SysycError> {
		val_decl.cond.accept(self)?;
        let cond_value = match val_decl.cond.get_attr(VALUE) {
            Some(Attr::Value(v)) => v.clone(),
            _ => {
                return Err(SysycError::LlvmSyntexError(format!(
                    "if cond has no value"
                )))
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
        self.funcemitter.as_mut().unwrap().visit_label(beginlabel.clone());
        val_decl.body.accept(self)?;
        match val_decl.then {
            Some(ref mut then_block) => {
                self.funcemitter.as_mut().unwrap().visit_jump_instr(exitlabel.clone());
                self.funcemitter.as_mut().unwrap().visit_label(skiplabel.clone());
                then_block.accept(self)?;
                self.funcemitter.as_mut().unwrap().visit_label(exitlabel.clone());
                Ok(())
            },
            None => {
                self.funcemitter.as_mut().unwrap().visit_label(skiplabel.clone());
                Ok(())
            }
        }
	}
	fn visit_while(&mut self, val_decl: &mut While) -> Result<(), SysycError> {
		let beginlabel = self.funcemitter.as_mut().unwrap().fresh_label();
        let looplabel = self.funcemitter.as_mut().unwrap().fresh_label();
        let breaklabel = self.funcemitter.as_mut().unwrap().fresh_label();
        self.funcemitter.as_mut().unwrap().openloop(breaklabel.clone(), looplabel.clone());
        self.funcemitter.as_mut().unwrap().visit_label(beginlabel.clone());
        val_decl.cond.accept(self)?;
        let cond_value = match val_decl.cond.get_attr(VALUE) {
            Some(Attr::Value(v)) => v.clone(),
            _ => {
                return Err(SysycError::LlvmSyntexError(format!(
                    "while cond has no value"
                )))
            }
        };
        let beginlabel_for_jump_cond_instr = self.funcemitter.as_mut().unwrap().fresh_label();
        self.funcemitter.as_mut().unwrap().visit_jump_cond_instr(
            cond_value,
            beginlabel_for_jump_cond_instr.clone(),
            breaklabel.clone(),
        );
        self.funcemitter.as_mut().unwrap().visit_label(beginlabel_for_jump_cond_instr.clone());
        val_decl.body.accept(self)?;
        self.funcemitter.as_mut().unwrap().visit_label(looplabel.clone());
        self.funcemitter.as_mut().unwrap().visit_jump_instr(beginlabel.clone());
        self.funcemitter.as_mut().unwrap().visit_label(breaklabel.clone());
        Ok(())
	}
	fn visit_init_val_list(
		&mut self,
		val_decl: &mut InitValList,
	) -> Result<(), SysycError> {
		// 这里的attr来自visit_var_def
		let symbol_id = match val_decl.get_attr(SYMBOL_NUMBER) {
			Some(Attr::UIntValue(id)) => *id,
			_ => {
				return Err(SysycError::LlvmSyntexError(format!(
					"init val has no symbol"
				)))
			}
		};
		let symbol = self.data.var_symbols[symbol_id].clone();
		for init_val in &mut val_decl.val_list {
			// 需要递归地告诉内部的InitValList，这个InitValList是属于哪个数组的
			init_val.set_attr(SYMBOL_NUMBER, Attr::UIntValue(symbol_id));
            init_val.accept(self)?;
            match init_val.get_attr(VALUE) {
                // 有 Some 说明这个init_val不是一个InitValList
                Some(Attr::Value(v)) => {
                    let addr = match init_val.get_attr(INDEX) {
						Some(Attr::UIntValue(index)) => {
							llvm::llvmop::Value::Int(*index as i32)
						},
						_ => {
							return Err(SysycError::LlvmSyntexError(format!(
								"init val has no index"
							)))
						}
					};
					let temp = self.funcemitter.as_mut().unwrap().visit_gep_instr(Value::Temp(symbol.temp.as_ref().unwrap().clone()), addr);
					self.funcemitter.as_mut().unwrap().visit_store_instr(v.clone(), Value::Temp(temp));
                },
                // None 说明这个init_val是一个InitValList, 会被递归地调用，这里什么都不用做
                None => {},
                _ => {
                    return Err(SysycError::LlvmSyntexError(format!(
                        "init val has no value"
                    )))
                }
            };
        }
        Ok(())
	}
	fn visit_literal_float(
		&mut self,
		val_decl: &mut LiteralFloat,
	) -> Result<(), SysycError> {
		val_decl.set_attr(VALUE, Attr::Value(llvm::llvmop::Value::Float(val_decl.value)));
		Ok(())
	}
	fn visit_literal_int(
		&mut self,
		val_decl: &mut LiteralInt,
	) -> Result<(), SysycError> {
		val_decl.set_attr(VALUE, Attr::Value(llvm::llvmop::Value::Int(val_decl.value)));
		Ok(())
	}
	fn visit_lval(&mut self, val_decl: &mut Lval) -> Result<(), SysycError> {
		let id = match val_decl.get_attr(SYMBOL_NUMBER) {
			Some(Attr::UIntValue(id)) => *id,
			_ => {
				return Err(SysycError::LlvmSyntexError(format!(
					"lval {} has no symbol",
					val_decl.ident.clone()
				)))
			}
		};
		let symbol = self.data.var_symbols[id].clone();
		let compile_const_index = match val_decl.get_attr(COMPILE_CONST_INDEX) {
			Some(a) => Some(a.clone()),
			None => None,
		};
		if let Some(dim_list) = &mut val_decl.dim_list {
			// 是数组
			// 数组索引是编译期常量
			if let Some(Attr::UIntValue(index)) = compile_const_index{
				let addr = if let Some(temp) = &symbol.temp {
					Value::Temp(temp.clone())
				} else {
					return Err(SysycError::LlvmSyntexError(format!(
						"lval {} has no temp",
						val_decl.ident.clone()
					)))
				};
				let target = self.funcemitter.as_mut().unwrap().visit_gep_instr(addr, Value::Int(index as i32));
				val_decl.set_attr(ADDRESS, Attr::Value(llvm::llvmop::Value::Temp(target.clone())));
				let target_value = self.funcemitter.as_mut().unwrap().visit_load_instr(Value::Temp(target));
				val_decl.set_attr(VALUE, Attr::Value(llvm::llvmop::Value::Temp(target_value)));
			} else {
				// 不是编译器常量，需要计算
				// 初始值不会被用到
				let mut temp = Temp {
					name: "temp".to_string(),
					var_type: VarType::I32,
				};
				let len = dim_list.len();
				for (i, dim) in dim_list.iter_mut().enumerate() {
					dim.accept(self)?;
					let dim_value = match dim.get_attr(VALUE) {
						Some(Attr::Value(v)) => v.clone(),
						_ => {
							return Err(SysycError::LlvmSyntexError(format!(
								"dim of lval has no value"
							)))
						}
					};
					if i == 0 {
						temp = self.funcemitter.as_mut().unwrap().visit_arith_instr(dim_value, llvm::llvmop::ArithOp::Mul, Value::Int(symbol.tp.dims[i+1] as i32))
					}else if i != len - 1{
						temp = self.funcemitter.as_mut().unwrap().visit_arith_instr(Value::Temp(temp), llvm::llvmop::ArithOp::Add, dim_value);
						temp = self.funcemitter.as_mut().unwrap().visit_arith_instr(Value::Temp(temp), llvm::llvmop::ArithOp::Mul, Value::Int(symbol.tp.dims[i+1] as i32))
					}else{
						temp = self.funcemitter.as_mut().unwrap().visit_arith_instr(Value::Temp(temp), llvm::llvmop::ArithOp::Add, dim_value);
					}
				}
				let addr = if let Some(temp) = &symbol.temp {
					Value::Temp(temp.clone())
				} else {
					return Err(SysycError::LlvmSyntexError(format!(
						"lval {} has no temp",
						val_decl.ident.clone()
					)))
				};
				let target = self.funcemitter.as_mut().unwrap().visit_gep_instr(addr, Value::Temp(temp));
				val_decl.set_attr(ADDRESS, Attr::Value(llvm::llvmop::Value::Temp(target.clone())));
				let target_value = self.funcemitter.as_mut().unwrap().visit_load_instr(Value::Temp(target));
				val_decl.set_attr(VALUE, Attr::Value(llvm::llvmop::Value::Temp(target_value)));
			}
		}else{
			// 是标量
			let mut tmp = if let Some(temp) = &symbol.temp {
				temp.clone()
			} else {
				return Err(SysycError::LlvmSyntexError(format!(
					"lval {} has no temp",
					val_decl.ident.clone()
				)))
			};
			val_decl.set_attr(ADDRESS, Attr::Value(llvm::llvmop::Value::Temp(tmp.clone())));
			tmp = self.funcemitter.as_mut().unwrap().visit_load_instr(Value::Temp(tmp));
			val_decl.set_attr(VALUE, Attr::Value(llvm::llvmop::Value::Temp(tmp)));
		}
		self.data.var_symbols[id] = symbol;
		Ok(())
	}
}
