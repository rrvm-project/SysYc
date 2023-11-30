use std::collections::HashMap;

use ast::{tree::*, Visitor};
use attr::Attrs;
use llvm::{llvmop::*, Value, VarType::*, *};
use rrvm::{
	cfg::link_cfg,
	program::{LlvmFunc, LlvmProgram},
	LlvmCFG,
};
use rrvm_symbol::{FuncSymbol, VarSymbol};
use utils::{errors::Result, Label, SysycError::TypeError};
use value::{BinaryOp, FuncRetType, UnaryOp, VarType};

use crate::{symbol_table::SymbolTable, utils::*};

pub struct IRGenerator {
	pub program: LlvmProgram,
	pub stack: Vec<(LlvmCFG, Option<Value>, Option<Value>)>,
	pub mgr: TempManager,
	pub total: i32,
	pub symbol_table: SymbolTable,
	pub ret_type: FuncRetType,
}

impl Default for IRGenerator {
	fn default() -> Self {
		Self::new()
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
	fn visit_init_val_list(&mut self, _node: &mut InitValList) -> Result<()> {
		// TODO: solve init_val_list
		todo!("I don't know how to solve this");
	}
	fn visit_variable(&mut self, node: &mut Variable) -> Result<()> {
		let now: LlvmCFG = self.new_cfg();
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
		let now: LlvmCFG = self.new_cfg();
		self.stack.push((now, Some(node.value.into()), None));
		Ok(())
	}
	fn visit_literal_float(&mut self, node: &mut LiteralFloat) -> Result<()> {
		let now: LlvmCFG = self.new_cfg();
		self.stack.push((now, Some(node.value.into()), None));
		Ok(())
	}
	fn visit_binary_expr(&mut self, node: &mut BinaryExpr) -> Result<()> {
		use BinaryOp::*;
		node.lhs.accept(self)?;
		let (mut lcfg, lhs_val, lhs_addr) = self.stack.pop().unwrap();
		self.symbol_table.push();
		node.rhs.accept(self)?;
		let (mut rcfg, rhs_val, rhs_addr) = self.stack.pop().unwrap();
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
	fn visit_while(&mut self, _node: &mut While) -> Result<()> {
		// TODO: while
		todo!();
	}
	fn visit_continue(&mut self, _node: &mut Continue) -> Result<()> {
		/*
		 这玩意本质是 goto 啊，咋处理来着
		*/
		// TODO: continue
		todo!();
	}
	fn visit_break(&mut self, _node: &mut Break) -> Result<()> {
		// TODO: break
		todo!();
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
