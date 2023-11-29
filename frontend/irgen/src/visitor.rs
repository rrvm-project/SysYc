#![allow(unused)]

use std::{
	cmp::max,
	collections::{HashMap, HashSet},
};

use ast::{tree::*, Visitor};
use attr::Attrs;
use llvm::{
	llvmop::{ArithOp, CompKind, CompOp, ConvertOp},
	llvmvar::{self},
	Value,
	VarType::*,
	*,
};
use rrvm::{
	cfg::{link_basic_block, link_cfg, CFG},
	program::{LlvmFunc, LlvmProgram, RrvmProgram},
	LlvmCFG,
};
use rrvm_symbol::{manager::SymbolManager, FuncSymbol, Symbol, VarSymbol};
use utils::{
	errors::Result,
	Label,
	SysycError::{LlvmvGenError, TypeError},
};
use value::{
	calc_type::{to_rval, type_binaryop},
	BType, BinaryOp, FuncRetType, FuncType, UnaryOp, VarType,
};

use crate::{
	symbol_table::{SymbolTable, Table},
	utils::*,
};

pub struct IRGenerator {
	program: LlvmProgram,
	stack: Vec<(LlvmCFG, Option<Value>, Option<Value>)>,
	mgr: TempManager,
	total: i32,
	symbol_table: SymbolTable,
	ret_type: FuncRetType,
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
			stack: Vec::new(),
			total: 0,
			mgr: TempManager::new(),
			symbol_table: SymbolTable::new(),
			ret_type: FuncRetType::Void,
		}
	}
	pub fn to_rrvm(mut self, program: &mut Program) -> Result<LlvmProgram> {
		program.accept(&mut self)?;
		Ok(self.program)
	}
	fn type_conv(
		&mut self,
		value: Value,
		target: llvm::VarType,
		cfg: &mut LlvmCFG,
	) -> Value {
		use llvmop::ConvertOp::*;
		if target == value.get_type() {
			return value;
		}
		let (from_type, to_type, op) = match target {
			I32 => (F32, I32, Float2Int),
			F32 => (I32, F32, Int2Float),
			_ => unreachable!(),
		};
		match (target, &value) {
			(F32, Value::Int(v)) => Value::Float(*v as f32),
			(I32, Value::Float(v)) => Value::Int(*v as i32),
			(_, Value::Temp(temp)) => {
				let target = self.mgr.new_temp(to_type, false);
				let instr = Box::new(ConvertInstr {
					op,
					target: target.clone(),
					from_type,
					lhs: temp.clone().into(),
					to_type,
				});
				cfg.get_exit().borrow_mut().push(instr);
				target.into()
			}
			_ => unreachable!(),
		}
	}
	fn solve(
		&mut self,
		val: Option<Value>,
		addr: Option<Value>,
		cfg: &mut LlvmCFG,
	) -> Value {
		match val {
			Some(value) => value,
			None => {
				let var_type = addr.as_ref().unwrap().deref_type();
				let temp = self.mgr.new_temp(var_type, false);
				let instr = Box::new(LoadInstr {
					target: temp.clone(),
					var_type,
					addr: addr.unwrap(),
				});
				cfg.get_exit().borrow_mut().push(instr);
				temp.into()
			}
		}
	}
	fn new_cfg(&mut self) -> LlvmCFG {
		let out = CFG::new(self.total);
		self.total += 1;
		out
	}
	pub fn fold_cfgs(&mut self, cfgs: Vec<LlvmCFG>) -> LlvmCFG {
		cfgs
			.into_iter()
			.reduce(|mut acc, mut v| {
				link_cfg(&mut acc, &mut v);
				acc.append(v);
				acc
			})
			.unwrap_or_else(|| self.new_cfg())
	}
	pub fn if_then_else(
		&mut self,
		mut cond: LlvmCFG,
		cond_val: Value,
		mut cfg1: LlvmCFG,
		diff1: Table,
		mut cfg2: LlvmCFG,
		diff2: Table,
	) -> LlvmCFG {
		let mut exit = self.new_cfg();
		let keys = diff1
			.keys()
			.chain(diff2.keys())
			.cloned()
			.collect::<HashSet<_>>()
			.into_iter();
		fn get_val(id: i32, now: &Table, default: &SymbolTable) -> Value {
			now.get(&id).map_or_else(|| default.get(&id), |v| v.clone())
		}
		for key in keys {
			let val1 = get_val(key, &diff1, &self.symbol_table);
			let val2 = get_val(key, &diff1, &self.symbol_table);
			let var_type = val1.get_type();
			let temp = self.mgr.new_temp(var_type, false);
			let instr = PhiInstr {
				target: temp.clone(),
				var_type,
				source: vec![(val1, cfg1.exit_label()), (val2, cfg2.exit_label())],
			};
			exit.get_exit().borrow_mut().push_phi(instr);
			self.symbol_table.set(key, temp.into());
		}
		let instr = Box::new(JumpCondInstr {
			var_type: I32,
			cond: cond_val,
			target_true: cfg1.entry_label(),
			target_false: cfg2.entry_label(),
		});
		cond.get_exit().borrow_mut().set_jump(Some(instr));
		link_cfg(&mut cond, &mut cfg1);
		link_cfg(&mut cond, &mut cfg2);
		link_cfg(&mut cfg1, &mut exit);
		link_cfg(&mut cfg2, &mut exit);
		cond.append(cfg1);
		cond.append(cfg2);
		cond.append(exit);
		cond
	}
}

impl Visitor for IRGenerator {
	fn visit_program(&mut self, node: &mut Program) -> Result<()> {
		self.symbol_table.push();
		for v in node.functions.iter_mut() {
			v.accept(self)?;
			self.total = 0;
		}
		Ok(())
	}
	fn visit_func_decl(&mut self, node: &mut FuncDecl) -> Result<()> {
		self.symbol_table.push();
		self.ret_type = node.ret_type;
		let mut params = Vec::new();
		for param in node.formal_params.iter_mut() {
			param.accept(self)?;
		}
		for param in node.formal_params.iter_mut() {
			let symbol: VarSymbol = param.get_attr("symbol").unwrap().into();
			params.push(self.symbol_table.get(&symbol.id));
		}
		node.block.accept(self)?;
		let (mut cfg, _, _) = self.stack.pop().unwrap();
		let var_type = func_type_convert(&node.ret_type);
		cfg.blocks.iter().for_each(|v| v.borrow_mut().gen_jump(var_type));
		cfg.sort();
		self.program.funcs.push(LlvmFunc::new(
			cfg,
			node.ident.clone(),
			var_type,
			params,
		));
		self.symbol_table.pop();
		Ok(())
	}
	fn visit_var_def(&mut self, node: &mut VarDef) -> Result<()> {
		let symbol: VarSymbol = node.get_attr("symbol").unwrap().into();
		let var_type = type_convert(&symbol.var_type);
		if let Some(init) = node.init.as_mut() {
			init.accept(self)?;
			let (mut cfg, value, _) = self.stack.pop().unwrap();
			if symbol.var_type.is_array() {
				// TODO: solve array init value list
				todo!()
			} else {
				let value = self.type_conv(value.unwrap(), var_type, &mut cfg);
				self.symbol_table.set(symbol.id, value);
			};
			self.stack.push((cfg, None, None));
		} else {
			let cfg: LlvmCFG = self.new_cfg();
			if symbol.var_type.is_array() {
				let length: usize = symbol.var_type.dims.iter().product();
				let temp = self.mgr.new_temp(var_type, false);
				self.symbol_table.set(symbol.id, temp.clone().into());
				let instr = Box::new(AllocInstr {
					target: temp.clone(),
					length: (length as i32).into(),
					var_type,
				});
				cfg.get_entry().borrow_mut().push(instr);
			} else {
				self.symbol_table.set(symbol.id, var_type.default_value());
			};
			self.stack.push((cfg, None, None));
		}
		Ok(())
	}
	fn visit_var_decl(&mut self, node: &mut VarDecl) -> Result<()> {
		let mut cfgs = Vec::new();
		for var_def in node.defs.iter_mut() {
			var_def.accept(self)?;
			cfgs.push(self.stack.pop().unwrap().0);
		}
		let cfg = self.fold_cfgs(cfgs);
		self.stack.push((cfg, None, None));
		Ok(())
	}
	fn visit_init_val_list(&mut self, node: &mut InitValList) -> Result<()> {
		// TODO: solve init_val_list
		todo!("I don't know how to solve this");
		for val in node.val_list.iter_mut() {
			val.accept(self)?;
		}
		Ok(())
	}
	fn visit_variable(&mut self, node: &mut Variable) -> Result<()> {
		let mut now: LlvmCFG = self.new_cfg();
		let symbol: VarSymbol = node.get_attr("symbol").unwrap().into();
		let temp = self.symbol_table.get(&symbol.id);
		if !symbol.var_type.is_array() {
			self.stack.push((now, Some(temp), None));
		} else {
			self.stack.push((now, None, Some(temp)));
		}
		Ok(())
	}
	fn visit_literal_int(&mut self, node: &mut LiteralInt) -> Result<()> {
		let mut now: LlvmCFG = self.new_cfg();
		self.stack.push((now, Some(node.value.into()), None));
		Ok(())
	}
	fn visit_literal_float(&mut self, node: &mut LiteralFloat) -> Result<()> {
		let mut now: LlvmCFG = self.new_cfg();
		self.stack.push((now, Some(node.value.into()), None));
		Ok(())
	}
	fn visit_binary_expr(&mut self, node: &mut BinaryExpr) -> Result<()> {
		use BinaryOp::*;
		node.lhs.accept(self);
		let (mut lcfg, lhs_val, lhs_addr) = self.stack.pop().unwrap();
		self.symbol_table.push();
		node.rhs.accept(self);
		let (mut rcfg, rhs_val, rhs_addr) = self.stack.pop().unwrap();
		let lhs_type: VarType = node.lhs.get_attr("type").unwrap().into();
		let rhs_type: VarType = node.lhs.get_attr("type").unwrap().into();
		let type_t: VarType = node.get_attr("type").unwrap().into();
		let var_type = type_convert(&type_t);
		let (cfg, ret_val, ret_addr) = match node.op {
			Assign => {
				let rhs_val = self.solve(rhs_val, rhs_addr, &mut rcfg);
				let val = self.type_conv(rhs_val, var_type, &mut rcfg);
				if let Some(addr) = &lhs_addr {
					let instr = Box::new(StoreInstr {
						value: val.clone(),
						addr: addr.clone(),
					});
					rcfg.get_exit().borrow_mut().push(instr);
				}
				if let Some(symbol) = node.lhs.get_attr("symbol") {
					let symbol: VarSymbol = symbol.into();
					self.symbol_table.set(symbol.id, val.clone());
				}
				link_cfg(&mut lcfg, &mut rcfg);
				lcfg.append(rcfg);
				self.symbol_table.pop();
				(lcfg, Some(val), lhs_addr)
			}
			IDX => {
				let rhs_val = self.solve(rhs_val, rhs_addr, &mut rcfg);
				let rhs = self.type_conv(rhs_val, I32, &mut rcfg);
				let offset = self.mgr.new_temp(I32, false);
				let instr = Box::new(ArithInstr {
					target: offset.clone(),
					lhs: rhs,
					var_type: I32,
					op: ArithOp::Mul,
					rhs: type_t.size().into(),
				});
				rcfg.get_exit().borrow_mut().push(instr);
				let var_type = lhs_addr.as_ref().unwrap().get_type();
				let temp = self.mgr.new_temp(var_type, false);
				let instr = Box::new(GEPInstr {
					target: temp.clone(),
					var_type,
					addr: lhs_addr.unwrap(),
					offset: offset.into(),
				});
				rcfg.get_exit().borrow_mut().push(instr);
				link_cfg(&mut lcfg, &mut rcfg);
				lcfg.append(rcfg);
				self.symbol_table.pop();
				(lcfg, None, Some(temp.into()))
			}
			_ => {
				let lhs_val = self.solve(lhs_val, lhs_addr, &mut lcfg);
				let rhs_val = self.solve(rhs_val, rhs_addr, &mut rcfg);
				let lhs = self.type_conv(lhs_val, var_type, &mut lcfg);
				let rhs = self.type_conv(rhs_val, var_type, &mut rcfg);
				match node.op {
					Add | Sub | Mul | Div | Mod => {
						let op = to_arith(node.op, var_type);
						let temp = self.mgr.new_temp(var_type, false);
						let instr = Box::new(ArithInstr {
							target: temp.clone(),
							op,
							lhs,
							rhs,
							var_type,
						});
						rcfg.get_exit().borrow_mut().push(instr);
						link_cfg(&mut lcfg, &mut rcfg);
						lcfg.append(rcfg);
						self.symbol_table.pop();
						(lcfg, Some(temp.into()), None)
					}
					LT | LE | GE | GT | EQ | NE => {
						let op = to_comp(node.op, var_type);
						let temp = self.mgr.new_temp(var_type, false);
						let instr = Box::new(CompInstr {
							kind: get_comp_kind(var_type),
							target: temp.clone(),
							op,
							lhs,
							rhs,
							var_type,
						});
						rcfg.get_exit().borrow_mut().push(instr);
						link_cfg(&mut lcfg, &mut rcfg);
						lcfg.append(rcfg);
						self.symbol_table.pop();
						(lcfg, Some(temp.into()), None)
					}
					LOr | LAnd => {
						/* TODO: 逻辑运算的 i1 类型
							 这里有 bug，返回右式的时候会返回原始值而不是 bool
							要解决的话需要加一个类型 bool，但是在逻辑计算的过程中始终不将原始值转成 bool
							只有当需要体现真实值特征的时候转 bool
							现在的实现忽略了相关判断，为了更高的运行效率。
							事实上 Sysy2022 的文法中不包含这种情况，测例里有再改。
						*/
						let source = vec![
							(((node.op == LOr) as i32).into(), lcfg.exit_label()),
							(rhs, rcfg.exit_label()),
						];
						let diff = self.symbol_table.drop();
						let cfg_empty = self.new_cfg();
						let diff_empty = HashMap::new();
						let cfg = if node.op == LAnd {
							self.if_then_else(lcfg, lhs, rcfg, diff, cfg_empty, diff_empty)
						} else {
							self.if_then_else(lcfg, lhs, cfg_empty, diff_empty, rcfg, diff)
						};
						let temp = self.mgr.new_temp(I32, false);
						let instr = PhiInstr {
							target: temp.clone(),
							var_type,
							source,
						};
						cfg.get_exit().borrow_mut().push_phi(instr);
						(cfg, Some(temp.into()), None)
					}
					_ => unreachable!(),
				}
			}
		};
		self.stack.push((cfg, ret_val, ret_addr));
		Ok(())
	}
	fn visit_unary_expr(&mut self, node: &mut UnaryExpr) -> Result<()> {
		let type_t: VarType = node.get_attr("type").unwrap().into();
		let var_type = type_convert(&type_t);
		node.rhs.accept(self)?;
		let (mut cfg, val, addr) = self.stack.pop().unwrap();
		let temp = self.solve(val, addr, &mut cfg);
		match node.op {
			UnaryOp::Plus => self.stack.push((cfg, Some(temp), None)),
			UnaryOp::Neg => {
				let op = to_arith(BinaryOp::Sub, var_type);
				let target = self.mgr.new_temp(var_type, false);
				let instr = Box::new(ArithInstr {
					target: target.clone(),
					op,
					lhs: var_type.default_value(),
					var_type,
					rhs: temp,
				});
				cfg.get_exit().borrow_mut().push(instr);
				self.stack.push((cfg, Some(target.into()), None));
			}
			UnaryOp::Not => {
				let target = self.mgr.new_temp(var_type, false);
				let instr = Box::new(CompInstr {
					kind: CompKind::Icmp,
					target: target.clone(),
					op: CompOp::EQ,
					lhs: var_type.default_value(),
					var_type,
					rhs: temp,
				});
				cfg.get_exit().borrow_mut().push(instr);
				self.stack.push((cfg, Some(target.into()), None));
			}
		}
		Ok(())
	}
	fn visit_func_call(&mut self, node: &mut FuncCall) -> Result<()> {
		let symbol: FuncSymbol = node.get_attr("func_symbol").unwrap().into();
		let mut cfgs = Vec::new();
		let mut params = Vec::new();
		let (ret_type, params_type) = symbol.var_type;
		for (param, type_t) in node.params.iter_mut().zip(params_type.iter()) {
			param.accept(self)?;
			let (mut cfg, val, addr) = self.stack.pop().unwrap();
			let var_type = type_convert(type_t);
			let val = self.solve(val, addr, &mut cfg);
			let val = self.type_conv(val, var_type, &mut cfg);
			cfgs.push(cfg);
			params.push((var_type, val));
		}
		let var_type = func_type_convert(&ret_type);
		let cfg = self.fold_cfgs(cfgs);
		let temp = self.mgr.new_temp(var_type, false);
		let instr = Box::new(CallInstr {
			target: temp.clone(),
			var_type,
			func: Label::new(symbol.ident),
			params,
		});
		cfg.get_exit().borrow_mut().push(instr);
		self.stack.push((cfg, Some(temp.into()), None));
		Ok(())
	}
	fn visit_formal_param(&mut self, node: &mut FormalParam) -> Result<()> {
		let symbol: VarSymbol = node.get_attr("symbol").unwrap().into();
		let temp = self.mgr.new_temp(type_convert(&symbol.var_type), false);
		self.symbol_table.set(symbol.id, temp.into());
		Ok(())
	}
	fn visit_block(&mut self, node: &mut Block) -> Result<()> {
		let mut cfgs = Vec::new();
		for stmt in node.stmts.iter_mut() {
			stmt.accept(self)?;
			cfgs.push(self.stack.pop().unwrap().0);
		}
		let cfg = self.fold_cfgs(cfgs);
		self.stack.push((cfg, None, None));
		Ok(())
	}
	fn visit_if(&mut self, node: &mut If) -> Result<()> {
		node.cond.accept(self)?;
		let (mut cond, cond_val, cond_addr) = self.stack.pop().unwrap();
		let cond_val = self.solve(cond_val, cond_addr, &mut cond);
		self.symbol_table.push();
		node.body.accept(self)?;
		let (cfg1, _, _) = self.stack.pop().unwrap();
		let diff1 = self.symbol_table.drop();
		let (cfg2, diff2) = if let Some(then) = node.then.as_mut() {
			self.symbol_table.push();
			then.accept(self)?;
			let (cfg, _, _) = self.stack.pop().unwrap();
			let diff = self.symbol_table.drop();
			(cfg, diff)
		} else {
			(self.new_cfg(), HashMap::new())
		};
		let cfg = self.if_then_else(cond, cond_val, cfg1, diff1, cfg2, diff2);
		self.stack.push((cfg, None, None));
		Ok(())
	}
	fn visit_while(&mut self, node: &mut While) -> Result<()> {
		todo!();
		node.cond.accept(self)?;
		node.body.accept(self)?;
		Ok(())
	}
	fn visit_continue(&mut self, node: &mut Continue) -> Result<()> {
		/*
		 这玩意本质是 goto 啊，咋处理来着
		*/
		todo!();
		Ok(())
	}
	fn visit_break(&mut self, node: &mut Break) -> Result<()> {
		todo!();
		Ok(())
	}
	fn visit_return(&mut self, node: &mut Return) -> Result<()> {
		if let Some(val) = &mut node.value {
			if self.ret_type == FuncRetType::Void {
				return Err(TypeError(
					"return with a value, in function returning void".to_string(),
				));
			}
			val.accept(self)?;
			let (mut cfg, val, addr) = self.stack.pop().unwrap();
			let var_type = func_type_convert(&self.ret_type);
			let val = self.solve(val, addr, &mut cfg);
			let val = self.type_conv(val, var_type, &mut cfg);
			let instr = Box::new(RetInstr { value: Some(val) });
			cfg.get_exit().borrow_mut().set_jump(Some(instr));
			self.stack.push((cfg, None, None));
		} else {
			if self.ret_type != FuncRetType::Void {
				return Err(TypeError(
					"with no value, in function returning non-void".to_string(),
				));
			}
			let cfg = self.new_cfg();
			let instr = Box::new(RetInstr { value: None });
			cfg.get_exit().borrow_mut().set_jump(Some(instr));
			self.stack.push((cfg, None, None));
		}
		Ok(())
	}
}
