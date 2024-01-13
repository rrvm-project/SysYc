use super::RemoveUselessPhis;
use crate::RrvmOptimizer;
use llvm::{LlvmInstrTrait, Temp, Value};
use rrvm::{program::LlvmProgram, LlvmCFG};
use std::collections::{HashMap, HashSet};
use utils::{errors::Result, UseTemp};

impl RrvmOptimizer for RemoveUselessPhis {
	fn new() -> Self {
		Self {}
	}
	fn apply(self, program: &mut LlvmProgram) -> Result<bool> {
		fn solve(cfg: &mut LlvmCFG) -> bool {
			let mut flag = false;
			let mut temp_mapper = HashMap::new();
			for block in cfg.blocks.iter_mut() {
				let block = &mut block.borrow_mut();
				block.phi_instrs.retain(|instr| {
					let target = instr.get_write().unwrap();
					let mut sources = HashSet::new();
					for (value, _) in instr.source.iter() {
						if value.unwrap_temp().map_or(true, |v| v != target) {
							sources.insert(value.clone());
						}
					}
					sources.len() != 1 || {
						let temp = sources.into_iter().next().unwrap();
						temp_mapper.insert(target, temp);
						false
					}
				});
			}
			flag |= !temp_mapper.is_empty();
			fn fix(k: &Temp, v: Value, mapper: &mut HashMap<Temp, Value>) -> Value {
				if let Some(temp) = v.unwrap_temp() {
					if let Some(val) = mapper.get(&temp).cloned() {
						let val = fix(&temp, val, mapper);
						*mapper.get_mut(k).unwrap() = val.clone();
						return val;
					}
				}
				v
			}
			if !temp_mapper.is_empty() {
				for (k, v) in temp_mapper.clone() {
					let _ = fix(&k, v, &mut temp_mapper);
				}
				for block in cfg.blocks.iter_mut() {
					for instr in block.borrow_mut().instrs.iter_mut() {
						instr.map_temp(&temp_mapper);
					}
					for instr in block.borrow_mut().phi_instrs.iter_mut() {
						instr.map_temp(&temp_mapper);
					}
					for instr in block.borrow_mut().jump_instr.iter_mut() {
						instr.map_temp(&temp_mapper);
					}
				}
			}
			flag
		}
		program.analysis();
		let flag = program
			.funcs
			.iter_mut()
			.fold(false, |last, func| solve(&mut func.cfg) || last);
		Ok(flag)
	}
}
