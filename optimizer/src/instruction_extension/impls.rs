use std::collections::HashMap;

use rrvm::{
	dominator::LlvmDomTree,
	program::{LlvmFunc, LlvmProgram},
	LlvmNode,
};

use crate::{metadata::MetaData, RrvmOptimizer};

use llvm::{ArithOp::*, CompOp, LlvmInstr, LlvmInstrVariant::*, LlvmTemp};

use llvm::VarType::*;

use super::InstructionExtension;

use utils::Result;

struct Solver {
	dom_tree: LlvmDomTree,
	comp_mapper: HashMap<LlvmTemp, llvm::CompInstr>,
}

impl Solver {
	pub fn new(func: &LlvmFunc) -> Self {
		Self {
			dom_tree: LlvmDomTree::new(&func.cfg, false),
			comp_mapper: HashMap::new(),
		}
	}
	pub fn dfs(&mut self, node: LlvmNode) {
		let children = self.dom_tree.get_children(node.borrow().id).clone();
		for instr in node.borrow().instrs.iter() {
			if let CompInstr(instr) = instr.get_variant() {
				self.comp_mapper.insert(instr.target.clone(), instr.clone());
			}
		}
		for v in children {
			self.dfs(v);
		}
	}
	pub fn get_comp_instr(&self, temp: &LlvmTemp) -> Option<&llvm::CompInstr> {
		self.comp_mapper.get(temp)
	}
}

impl RrvmOptimizer for InstructionExtension {
	fn new() -> Self {
		Self {}
	}

	fn apply(
		self,
		program: &mut LlvmProgram,
		_metadata: &mut MetaData,
	) -> Result<bool> {
		fn solve(func: &LlvmFunc) -> bool {
			let mut flag = false;
			let mut solver = Solver::new(func);
			solver.dfs(func.cfg.get_entry());
			// let func_data = metadata.get_func_data(&func.name);
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
					if !prev1.instrs.is_empty() || !prev2.instrs.is_empty() {
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

					if let Some(comp_instr) = solver
						.get_comp_instr(&father_jump_instr.cond.unwrap_temp().unwrap())
					{
						let mut failed = false;
						let mut instrs: Vec<LlvmInstr> = Vec::new();
						match comp_instr.op {
							CompOp::SLT | CompOp::SLE => {
								for instr in block.phi_instrs.iter() {
									let value1 = instr.get_value(&prev1.label()).unwrap();
									let value2 = instr.get_value(&prev2.label()).unwrap();
									if value1 == comp_instr.lhs && value2 == comp_instr.rhs {
										instrs.push(Box::new(llvm::ArithInstr {
											target: instr.target.clone(),
											op: Min,
											var_type: I32,
											lhs: value1.clone(),
											rhs: value2.clone(),
										}))
									} else if value1 == comp_instr.rhs && value2 == comp_instr.lhs
									{
										instrs.push(Box::new(llvm::ArithInstr {
											target: instr.target.clone(),
											op: Max,
											var_type: I32,
											lhs: value1.clone(),
											rhs: value2.clone(),
										}))
									} else {
										failed = true;
										break;
									}
								}
							}
							CompOp::SGT | CompOp::SGE => {
								for instr in block.phi_instrs.iter() {
									let value1 = instr.get_value(&prev1.label()).unwrap();
									let value2 = instr.get_value(&prev2.label()).unwrap();
									if value1 == comp_instr.lhs && value2 == comp_instr.rhs {
										instrs.push(Box::new(llvm::ArithInstr {
											target: instr.target.clone(),
											op: Max,
											var_type: I32,
											lhs: value1.clone(),
											rhs: value2.clone(),
										}))
									} else if value1 == comp_instr.rhs && value2 == comp_instr.lhs
									{
										instrs.push(Box::new(llvm::ArithInstr {
											target: instr.target.clone(),
											op: Min,
											var_type: I32,
											lhs: value1.clone(),
											rhs: value2.clone(),
										}))
									} else {
										failed = true;
										break;
									}
								}
							}
							_ => {
								failed = true;
							}
						}
						if !failed {
							flag = true;
							block.phi_instrs.clear();
							block.instrs.splice(0..0, instrs);
							father.jump_instr = Some(Box::new(llvm::JumpInstr {
								target: block.label(),
							}));
							father.succ = vec![node.clone()];
						}
					}
				}
			}
			flag
		}

		Ok(program.funcs.iter().fold(false, |last, func| solve(func) || last))
	}
}
