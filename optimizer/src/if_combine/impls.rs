use std::collections::HashSet;

use rrvm::{
	dominator::LlvmDomTree,
	program::{LlvmFunc, LlvmProgram},
	LlvmNode,
};

use crate::{metadata::MetaData, RrvmOptimizer};

use llvm::{
	compute_two_boolean, LlvmInstr, LlvmInstrVariant::*, LlvmTemp,
	LlvmTempManager, Value,
};

use llvm::VarType::*;

use super::IfCombine;

use utils::Result;

struct Solver {
	dom_tree: LlvmDomTree,
	boolean: HashSet<LlvmTemp>,
}

impl Solver {
	pub fn new(func: &LlvmFunc) -> Self {
		Self {
			dom_tree: LlvmDomTree::new(&func.cfg, false),
			boolean: HashSet::new(),
		}
	}
	pub fn dfs(&mut self, node: LlvmNode) {
		let children = self.dom_tree.get_children(node.borrow().id).clone();
		for instr in node.borrow().instrs.iter() {
			match instr.get_variant() {
				ArithInstr(instr) => {
					use llvm::ArithOp::*;
					if matches!(instr.op, And | Or | Xor)
						&& self.is_boolean(&instr.lhs)
						&& self.is_boolean(&instr.rhs)
					{
						self.boolean.insert(instr.target.clone());
					}
				}
				CompInstr(instr) => {
					self.boolean.insert(instr.target.clone());
				}
				_ => {}
			}
		}
		for v in children {
			self.dfs(v);
		}
	}
	pub fn is_boolean(&self, temp: &Value) -> bool {
		match temp {
			Value::Int(v) => *v == 0 || *v == 1,
			Value::Temp(temp) => self.boolean.contains(temp),
			_ => false,
		}
	}

	pub fn combine_value(
		&self,
		value1: Value,
		value2: Value,
		true_cond: &Value,
		instrs: &mut Vec<LlvmInstr>,
		mgr: &mut LlvmTempManager,
	) -> Value {
		fn combine_boolean(
			value1: Value,
			value2: Value,
			true_cond: &Value,
			instrs: &mut Vec<LlvmInstr>,
			mgr: &mut LlvmTempManager,
		) -> Value {
			let false_cond = mgr.new_temp(I32, false);
			instrs.push(Box::new(llvm::CompInstr {
				kind: llvm::CompKind::Icmp,
				target: false_cond.clone(),
				op: llvm::CompOp::EQ,
				lhs: true_cond.clone(),
				var_type: I32,
				rhs: 0.into(),
			}));
			let (lhs_val, lhs_instr) =
				compute_two_boolean(value1, true_cond.clone(), llvm::ArithOp::And, mgr);
			if let Some(instr) = lhs_instr {
				instrs.push(instr);
			}
			let (rhs_val, rhs_instr) = compute_two_boolean(
				value2,
				false_cond.clone().into(),
				llvm::ArithOp::And,
				mgr,
			);
			if let Some(instr) = rhs_instr {
				instrs.push(instr);
			}
			let (val, instr) =
				compute_two_boolean(lhs_val, rhs_val, llvm::ArithOp::Or, mgr);
			if let Some(instr) = instr {
				instrs.push(instr);
			}
			val
		}

		fn combine_i32(
			value1: Value,
			value2: Value,
			true_cond: &Value,
			instrs: &mut Vec<LlvmInstr>,
			mgr: &mut LlvmTempManager,
		) -> Value {
			let mask = mgr.new_temp(I32, false);
			instrs.push(Box::new(llvm::ArithInstr {
				op: llvm::ArithOp::Sub,
				target: mask.clone(),
				lhs: 0.into(),
				rhs: true_cond.clone(),
				var_type: I32,
			}));
			let diff = mgr.new_temp(I32, false);
			let masked_diff = mgr.new_temp(I32, false);
			instrs.push(Box::new(llvm::ArithInstr {
				op: llvm::ArithOp::Sub,
				target: diff.clone(),
				lhs: value1.clone(),
				rhs: value2.clone(),
				var_type: I32,
			}));
			instrs.push(Box::new(llvm::ArithInstr {
				op: llvm::ArithOp::And,
				target: masked_diff.clone(),
				lhs: diff.clone().into(),
				rhs: mask.clone().into(),
				var_type: I32,
			}));
			let target = mgr.new_temp(I32, false);
			instrs.push(Box::new(llvm::ArithInstr {
				op: llvm::ArithOp::Add,
				target: target.clone(),
				lhs: value2.clone(),
				rhs: masked_diff.clone().into(),
				var_type: I32,
			}));
			target.into()
		}

		fn combine_f32(
			value1: Value,
			value2: Value,
			true_cond: &Value,
			instrs: &mut Vec<LlvmInstr>,
			mgr: &mut LlvmTempManager,
		) -> Value {
			let coef = mgr.new_temp(F32, false);
			instrs.push(Box::new(llvm::ConvertInstr {
				op: llvm::ConvertOp::Int2Float,
				target: coef.clone(),
				lhs: true_cond.clone(),
				var_type: F32,
			}));
			let diff = mgr.new_temp(F32, false);
			let masked_diff = mgr.new_temp(F32, false);
			instrs.push(Box::new(llvm::ArithInstr {
				op: llvm::ArithOp::Fsub,
				target: diff.clone(),
				lhs: value1.clone(),
				rhs: value2.clone(),
				var_type: F32,
			}));
			instrs.push(Box::new(llvm::ArithInstr {
				op: llvm::ArithOp::Fmul,
				target: masked_diff.clone(),
				lhs: diff.clone().into(),
				rhs: coef.clone().into(),
				var_type: F32,
			}));
			let target = mgr.new_temp(F32, false);
			instrs.push(Box::new(llvm::ArithInstr {
				op: llvm::ArithOp::Fadd,
				target: target.clone(),
				lhs: value2.clone(),
				rhs: masked_diff.clone().into(),
				var_type: F32,
			}));
			target.into()
		}

		if value1 == value2 {
			value1
		} else if self.is_boolean(&value1) && self.is_boolean(&value2) {
			combine_boolean(value1, value2, true_cond, instrs, mgr)
		} else if !value1.get_type().is_float() {
			combine_i32(value1, value2, true_cond, instrs, mgr)
		} else {
			combine_f32(value1, value2, true_cond, instrs, mgr)
		}
	}
}

impl RrvmOptimizer for IfCombine {
	fn new() -> Self {
		Self {}
	}

	fn apply(
		self,
		program: &mut LlvmProgram,
		metadata: &mut MetaData,
	) -> Result<bool> {
		fn solve(
			func: &LlvmFunc,
			metadata: &mut MetaData,
			mgr: &mut LlvmTempManager,
		) -> bool {
			let mut flag = false;
			let mut solver = Solver::new(func);
			solver.dfs(func.cfg.get_entry());
			let func_data = metadata.get_func_data(&func.name);
			for node in func.cfg.blocks.iter() {
				if node.borrow().prev.len() == 2 {
					let prev1 = node.borrow().prev[0].clone();
					let prev2 = node.borrow().prev[1].clone();
					if prev1.borrow().prev.len() != 1
						|| prev2.borrow().prev.len() != 1
						|| prev1.borrow().prev[0] != prev2.borrow().prev[0]
					{
						continue;
					}
					let prev1 = &mut prev1.borrow_mut();
					let prev2 = &mut prev2.borrow_mut();
					if prev1.instrs.is_empty() && prev2.instrs.is_empty() {
						continue;
					}
					let mut block = node.borrow_mut();
					let father = prev1.prev[0].clone();
					let father = &mut father.borrow_mut();

					// 1 为 true 2 为 false
					let father_jump_instr = if let JumpCondInstr(instr) =
						father.jump_instr.as_ref().unwrap().get_variant()
					{
						instr
					} else {
						unreachable!()
					};
					let (prev1, prev2) = {
						if father_jump_instr.target_true == prev1.label() {
							(prev1, prev2)
						} else {
							(prev2, prev1)
						}
					};

					let mut instrs = Vec::new();

					let mut failed = false;

					let mut iter1 = prev1.instrs.iter().peekable();
					let mut iter2 = prev2.instrs.iter().peekable();

					loop {
						match (iter1.peek(), iter2.peek()) {
							(Some(instr1), _) if !instr1.has_sideeffect() => {
								instrs.push(iter1.next().unwrap().clone_box());
							}
							(_, Some(instr2)) if !instr2.has_sideeffect() => {
								instrs.push(iter2.next().unwrap().clone_box());
							}
							(Some(_), Some(_)) => {
								match (
									iter1.next().unwrap().get_variant(),
									iter2.next().unwrap().get_variant(),
								) {
									(StoreInstr(instr1), StoreInstr(instr2)) => {
										if !func_data.value_euqal(&instr1.addr, &instr2.addr) {
											failed = true;
											break;
										}
										let value = solver.combine_value(
											instr1.value.clone(),
											instr2.value.clone(),
											&father_jump_instr.cond,
											&mut instrs,
											mgr,
										);
										instrs.push(Box::new(llvm::StoreInstr {
											addr: instr1.addr.clone(),
											value,
										}));
									}
									(CallInstr(instr1), CallInstr(instr2)) => {
										if instr1.func.name != instr2.func.name
											|| instr1.var_type != Void
										{
											failed = true;
											break;
										}
										let params = instr1
											.params
											.iter()
											.zip(instr2.params.iter())
											.map(|((var_type, param1), (_, param2))| {
												(
													*var_type,
													solver.combine_value(
														param1.clone(),
														param2.clone(),
														&father_jump_instr.cond,
														&mut instrs,
														mgr,
													),
												)
											})
											.collect();
										instrs.push(Box::new(llvm::CallInstr {
											target: instr1.target.clone(),
											var_type: instr1.var_type,
											func: instr1.func.clone(),
											params,
										}));
									}
									_ => {
										failed = true;
										break;
									}
								}
							}
							(None, None) => {
								break;
							}
							_ => {
								failed = true;
								break;
							}
						}
					}

					if failed {
						continue;
					}

					flag = true;
					prev1.instrs.clear();
					prev2.instrs.clear();
					block.phi_instrs.clear();
					block.instrs.splice(0..0, instrs);
					father.jump_instr = Some(Box::new(llvm::JumpInstr {
						target: block.label(),
					}));
					father.succ = vec![node.clone()];
				}
			}
			flag
		}

		Ok(program.funcs.iter().fold(false, |last, func| {
			solve(func, metadata, &mut program.temp_mgr) || last
		}))
	}
}
