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

struct Solver<'a> {
	dom_tree: LlvmDomTree,
	_metadata: &'a mut MetaData,
	boolean: HashSet<LlvmTemp>,
}

impl<'a> Solver<'a> {
	pub fn new(func: &LlvmFunc, metadata: &'a mut MetaData) -> Self {
		Self {
			dom_tree: LlvmDomTree::new(&func.cfg, false),
			_metadata: metadata,
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
				target: masked_diff.clone(),
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
			let diff = mgr.new_temp(I32, false);
			let masked_diff = mgr.new_temp(I32, false);
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
			let target = mgr.new_temp(I32, false);
			instrs.push(Box::new(llvm::ArithInstr {
				op: llvm::ArithOp::Fadd,
				target: masked_diff.clone(),
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
			let mut solver = Solver::new(func, metadata);
			solver.dfs(func.cfg.get_entry());
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
					if prev1.instrs.iter().any(|instr| instr.get_write().is_some())
						|| prev2.instrs.iter().any(|instr| instr.get_write().is_some())
						|| prev1.instrs.len() != prev2.instrs.len()
						|| prev1.instrs.is_empty()
					{
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
					for (instr1, instr2) in prev1.instrs.iter().zip(prev2.instrs.iter()) {
						match (instr1.get_variant(), instr2.get_variant()) {
							(StoreInstr(instr1), StoreInstr(instr2)) => {
								if instr1.addr != instr2.addr {
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
							_ => {
								failed = true;
								break;
							}
						}
					}

					if failed {
						continue;
					}

					if !block.phi_instrs.is_empty() {
						eprintln!("别急");
						continue;
					}

					eprintln!("if combine 成功");
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
