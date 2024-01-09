use crate::RrvmOptimizer;
use llvm::{ArithInstr, ArithOp, Value, VarType};
use rrvm::{program::LlvmProgram, LlvmCFG};
use utils::errors::Result;

use super::RemoveUselessPhis;

// 如果 phi 指令每一项的值都是一样的，则将phi替换为赋值指令（这里覆盖了只有一项的 phi 指令）

impl RrvmOptimizer for RemoveUselessPhis {
	fn new() -> Self {
		Self {}
	}
	fn apply(self, program: &mut LlvmProgram) -> Result<bool> {
		fn solve(cfg: &mut LlvmCFG) -> bool {
			let mut flag = false;
			for block in cfg.blocks.iter_mut() {
				let mut block = block.borrow_mut();
				let mut to_replace = Vec::new();
				for phi in block.phi_instrs.iter_mut() {
					if let Some(v) = phi.all_has_the_same_value() {
						to_replace.push((phi.target.clone(), v.clone()));
					}
				}
				block
					.phi_instrs
					.retain(|phi| !to_replace.iter().any(|(t, _)| t == &phi.target));
				for (t, v) in to_replace {
					let op;
					let rhs;
					if t.var_type == VarType::F32 {
						op = ArithOp::Fadd;
						rhs = Value::Float(0.0);
					} else {
						op = ArithOp::Add;
						rhs = Value::Int(0);
					}
					block.instrs.insert(
						0,
						Box::new(ArithInstr {
							target: t.clone(),
							op,
							var_type: t.var_type,
							lhs: v,
							rhs,
						}),
					);
					flag = true;
				}
			}
			flag
		}

		Ok(
			program
				.funcs
				.iter_mut()
				.fold(false, |last, func| solve(&mut func.cfg) || last),
		)
	}
}
