use std::{
	 cell::RefCell, collections::{HashMap, HashSet}, mem, rc::Rc
};

use llvm::{
	LlvmInstr, LlvmInstrVariant::{
		AllocInstr, ArithInstr, CallInstr, CompInstr, ConvertInstr, GEPInstr, JumpCondInstr, LoadInstr, PhiInstr, StoreInstr
	}, LlvmTemp, Value::{self, Temp}
};
use rrvm::{
	func,
	program::{LlvmFunc, LlvmProgram},
};

use super::{ast::AstNode, CalcCoef};
use crate::RrvmOptimizer;
use utils::errors::Result;
impl RrvmOptimizer for CalcCoef {
	fn new() -> Self {
		Self {}
	}
	fn apply(self, program: &mut LlvmProgram,metadata:&mut MetaData) -> Result<bool> {
		let old_len = program.funcs.len();
		let new_funcs: Vec<_> = mem::take(&mut program.funcs)
			.into_iter()
			.flat_map(|func| {
				if let Some(index) = can_calc(&func) {
					calc_coef(&func, index)
				} else {
					vec![func]
				}
			})
			.collect();
		program.funcs = new_funcs;
		Ok(old_len != program.funcs.len())
	}
}
fn can_calc(func: &LlvmFunc) -> Option<LlvmTemp> {
	if func.params.len() == 2 {
		let t = func.params.clone();
		// check if recursive call and has (every) branch based on the index parameter (此处加强了条件)
		let mut param: Option<llvm::LlvmTemp> = None;
		let mut has_recursion = false;
		for block in func.cfg.blocks.iter() {
			for instr in block.borrow().instrs.iter() {
				if let CallInstr(callinstr) = instr.get_variant() {
					if callinstr.func.name == func.name {
						has_recursion = true;
						break;
					}
				}
			}
		}
		// 判断是否纯函数，检查有无语句是 load/store
		for block in func.cfg.blocks.iter() {
			for instr in block.borrow().instrs.iter() {
				if let LoadInstr(_) | StoreInstr(_) = instr.get_variant() {
					return None;
				}
			}
		}
		if has_recursion {
			let mut jmp_conds = HashSet::new();
			for block in func.cfg.blocks.iter().rev() {
				// 找 branch 指令，找最后一次写他的指令 如果是参数是 param 或者是 param 和别人比较得到的结果就行
				let block = &block.borrow();
				for i in block.instrs.iter().rev() {
					if let JumpCondInstr(jmp_cond) = i.get_variant() {
						if let Value::Temp(t) = &jmp_cond.cond {
							if !func.params.contains(&jmp_cond.cond) {
								jmp_conds.insert(t.clone());
							}
						}
					}
					if let Some(t) = i.get_write() {
						if jmp_conds.contains(&t) {
							jmp_conds.remove(&t);
							// 进行检查
							if let CompInstr(compinstr) = i.get_variant() {
								if let Value::Temp(t) = &compinstr.lhs {
									if func.params.contains(&compinstr.lhs) {
										if let Some(ref p) = param {
											if *p != *t {
												return None;
											}
										} else {
											param = Some(t.clone());
										}
									}
								} else if let Value::Temp(t) = &compinstr.rhs {
									if func.params.contains(&compinstr.rhs) {
										if let Some(ref p) = param {
											if *p != *t {
												return None;
											}
										} else {
											param = Some(t.clone());
										}
									}
								}
							}
						}
					}
				}
			}
			if let Some(p) = param {
				return Some(p);
			}
		}
	}
	None
}
fn calc_coef(func: &LlvmFunc, index: LlvmTemp) -> Vec<LlvmFunc> {
	let data =
		func.params.iter().find(|x| **x != Value::Temp(index.clone())).unwrap();
	let mut tmp_map = HashMap::new();
	// 把俩参数先放进去
	if let Value::Temp(t) = data {
		tmp_map.insert(t.clone(), Rc::new(RefCell::new(AstNode::Value(data.clone()))));
	}
	tmp_map.insert(index.clone(), Rc::new(RefCell::new(AstNode::Value(Value::Temp(index.clone())))));
	//let mut ret_idx_map=HashMap::new();
	// tmp_map.insert(k, v)
	// 对函数基本块开始 dfs
	let mut dfs_stack = vec![func.cfg.blocks[0].clone()];
	while let Some(block) = dfs_stack.pop() {
		let block = block.borrow();
		// solve tmp values
		for instr in block.instrs.iter() {
			match instr.get_variant() {
				ArithInstr(instr) => {
					let lhs_node = {
						if let Value::Temp(t) = &instr.lhs {
							if tmp_map.contains_key(t) {
								tmp_map[t].clone()
							} else {
								Rc::new(RefCell::new(AstNode::Value(instr.lhs.clone())))
							}
						} else {
							Rc::new(RefCell::new(AstNode::Value(instr.lhs.clone())))
						}
					};
					let rhs = {
						if let Value::Temp(t) = &instr.rhs {
							if tmp_map.contains_key(t) {
								tmp_map[t].clone()
							} else {
								Rc::new(RefCell::new(AstNode::Value(instr.rhs.clone())))
							}
						} else {
							Rc::new(RefCell::new(AstNode::Value(instr.rhs.clone())))
						}
					};
					let target = instr.target.clone();
					tmp_map.insert(target, Rc::new(RefCell::new(AstNode::Expr((lhs_node, Box::new(instr.op.clone()), rhs)))));
				}
				CompInstr(instr)=>{
					// 和 ArithmeticInstr 一样
					let lhs_node = {
						if let Value::Temp(t) = &instr.lhs {
							if tmp_map.contains_key(t) {
								tmp_map[t].clone()
							} else {
								Rc::new(RefCell::new(AstNode::Value(instr.lhs.clone())))
							}
						} else {
							Rc::new(RefCell::new(AstNode::Value(instr.lhs.clone())))
						}
					};
					let rhs = {
						if let Value::Temp(t) = &instr.rhs {
							if tmp_map.contains_key(t) {
								tmp_map[t].clone()
							} else {
								Rc::new(RefCell::new(AstNode::Value(instr.rhs.clone())))
							}
						} else {
							Rc::new(RefCell::new(AstNode::Value(instr.rhs.clone())))
						}
					};
					let target = instr.target.clone();
					tmp_map.insert(target, Rc::new(RefCell::new(AstNode::Expr((lhs_node, Box::new(instr.op.clone()), rhs)))));
				}
				ConvertInstr(instr)=>{
					let lhs_node = {
						if let Value::Temp(t) = &instr.lhs {
							if tmp_map.contains_key(t) {
								tmp_map[t].clone()
							} else {
								Rc::new(RefCell::new(AstNode::Value(instr.lhs.clone())))
							}
						} else {
							Rc::new(RefCell::new(AstNode::Value(instr.lhs.clone())))
						}
					};
					let target = instr.target.clone();
					tmp_map.insert(target, Rc::new(RefCell::new(AstNode::ConvertNode(instr.to_type.clone(), lhs_node))));
				}
				PhiInstr(instr)=>{
					let mut phi_nodes = vec![];
					for (v, l) in instr.source.iter() {
						let node = {
							if let Value::Temp(t) = v {
								if tmp_map.contains_key(&t) {
									tmp_map[&t].clone()
								} else {
									Rc::new(RefCell::new(AstNode::Value(v.clone())))
								}
							} else {
								Rc::new(RefCell::new(AstNode::Value(v.clone())))
							}
						};
						phi_nodes.push((node, l.clone()));
					}
					tmp_map.insert(instr.target.clone(), Rc::new(RefCell::new(AstNode::PhiNode(phi_nodes))));
				}
				AllocInstr(instr)=>{
					tmp_map.insert(instr.target.clone(), Rc::new(RefCell::new(AstNode::AllocNode(instr.length.clone()))));
				}
				GEPInstr(instr)=>{
					let offset_node={
						if let Value::Temp(t) = &instr.offset {
							if tmp_map.contains_key(t) {
								tmp_map[t].clone()
							} else {
								Rc::new(RefCell::new(AstNode::Value(instr.offset.clone())))
							}
						} else {
							Rc::new(RefCell::new(AstNode::Value(instr.offset.clone())))
						}
					};
					let addr_node={
						if let Value::Temp(t) = &instr.addr {
							if tmp_map.contains_key(t) {
								tmp_map[t].clone()
							} else {
								Rc::new(RefCell::new(AstNode::Value(instr.addr.clone())))
							}
						} else {
							Rc::new(RefCell::new(AstNode::Value(instr.addr.clone())))
						}
					};
					tmp_map.insert(instr.target.clone(), Rc::new(RefCell::new(AstNode::GepNode(addr_node,offset_node))));
				}
				CallInstr(instr)=>{
					let mut arg_nodes:Vec<_>=instr.params.iter().map(|x|{
						if let Value::Temp(t)=&x.1{
							if tmp_map.contains_key(t){
								tmp_map[t].clone()
							}else{
								Rc::new(RefCell::new(AstNode::Value(x.1.clone())))
							}
						}else{
							Rc::new(RefCell::new(AstNode::Value(x.1.clone())))
						}
					}).collect();
					tmp_map.insert(instr.target.clone(), Rc::new(RefCell::new(AstNode::CallVal(instr.func.name.clone(),arg_nodes))));
				}
				_=>{}
			}
		}
		for i in block.succ.iter() {
			dfs_stack.push(i.clone());
		}
	}
	// dfs 完了 把每个函数 return 指令的结果展开
	Vec::new()
}
