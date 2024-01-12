use super::RemoveUselessPhis;
use crate::RrvmOptimizer;
use llvm::{LlvmInstrTrait, Temp};
use rrvm::{program::LlvmProgram, LlvmCFG};
use utils::errors::Result;

// 如果 phi 指令每一项的值都是一样的，则将phi替换为赋值指令（这里覆盖了只有一项的 phi 指令）

impl RrvmOptimizer for RemoveUselessPhis {
	fn new() -> Self {
		Self {}
	}
	fn apply(self, program: &mut LlvmProgram) -> Result<bool> {
		fn solve(cfg: &mut LlvmCFG) -> bool {
			let mut flag = false;
			let mut to_replace: Vec<(Temp, llvm::Value)> = Vec::new();
			for block in cfg.blocks.iter_mut() {
				let mut block = block.borrow_mut();
				for phi in block.phi_instrs.iter_mut() {
					if let Some(v) = phi.all_has_the_same_value() {
						// 防止出现链式消除的情况
						if v
							.unwrap_temp()
							.is_some_and(|temp| to_replace.iter().any(|(t, _)| *t == temp))
						{
							let new_v = to_replace
								.iter()
								.find_map(|(t, v_inner)| {
									if *t == v.unwrap_temp().unwrap() {
										Some(v_inner.clone())
									} else {
										None
									}
								})
								.unwrap();
							to_replace.push((phi.target.clone(), new_v.clone()));
						} else {
							to_replace.push((phi.target.clone(), v.clone()));
						}
					}
				}
			}
			// 替换应当是全局的
			for block in cfg.blocks.iter_mut() {
				let mut block = block.borrow_mut();
				block
					.phi_instrs
					.retain(|phi| !to_replace.iter().any(|(t, _)| t == &phi.target));
				for (t, v) in to_replace.iter() {
					block.phi_instrs.iter_mut().for_each(|instr| {
						instr.replace_read(t.clone(), v.clone());
					});
					block.instrs.iter_mut().for_each(|instr| {
						instr.replace_read(t.clone(), v.clone());
					});
					block.jump_instr.iter_mut().for_each(|instr| {
						instr.replace_read(t.clone(), v.clone());
					});
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
