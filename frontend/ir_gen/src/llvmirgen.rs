use ast::{tree::*, visitor::Visitor};
use attr::{Attr, Attrs};
use llvm::llvmvar::VarType;
use llvm::{func::LlvmFunc, llvmfuncemitter::LlvmFuncEmitter, LlvmProgram};
use namer::namer::SYMBOL_NUMBER;
use namer::utils::DataFromNamer;
use utils::SysycError;

static VALUE: &str = "value";
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
	fn visit_break(&mut self, val_decl: &mut Break) -> Result<(), SysycError> {
		todo!()
	}
	fn visit_continue(
		&mut self,
		val_decl: &mut Continue,
	) -> Result<(), SysycError> {
		todo!()
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
		todo!()
	}
	fn visit_while(&mut self, val_decl: &mut While) -> Result<(), SysycError> {
		todo!()
	}
	fn visit_init_val_list(
		&mut self,
		val_decl: &mut InitValList,
	) -> Result<(), SysycError> {
		todo!()
	}
	fn visit_literal_float(
		&mut self,
		val_decl: &mut LiteralFloat,
	) -> Result<(), SysycError> {
		todo!()
	}
	fn visit_literal_int(
		&mut self,
		val_decl: &mut LiteralInt,
	) -> Result<(), SysycError> {
		todo!()
	}
	fn visit_lval(&mut self, val_decl: &mut Lval) -> Result<(), SysycError> {
		todo!()
	}
	fn visit_unary_expr(
		&mut self,
		val_decl: &mut UnaryExpr,
	) -> Result<(), SysycError> {
		todo!()
	}
	fn visit_var_decl(
		&mut self,
		val_decl: &mut VarDecl,
	) -> Result<(), SysycError> {
		todo!()
	}
	fn visit_var_def(&mut self, val_decl: &mut VarDef) -> Result<(), SysycError> {
		todo!()
	}
}
