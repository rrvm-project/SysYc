use std::collections::{HashMap, HashSet, VecDeque};

use super::PureCheck;
use crate::RrvmOptimizer;
use rrvm::program::LlvmProgram;
use utils::errors::Result;

impl RrvmOptimizer for PureCheck {
	fn new() -> Self {
		Self {}
	}

	fn apply(self, program: &mut LlvmProgram) -> Result<bool> {
		let mut work_list: VecDeque<String> = VecDeque::new();
		let mut inverse_call_relation: HashMap<String, HashSet<String>> =
			HashMap::new();

		// program.funcs.iter_mut().map(|func| mark(func));
		for func in program.funcs.iter_mut() {
			for block in &func.cfg.blocks {
				for instr in &block.borrow().instrs {
					if let Some(item) = instr.external_resorce() {
						// func.external_resorce.insert(item);
						match item {
							utils::purity::ExternalResource::Call(callee) => {
								inverse_call_relation
									.entry(callee.clone())
									.or_default()
									.insert(func.name.clone());
							}
							v => {
								work_list.push_front(func.name.clone());
								func.external_resorce.insert(v);
							}
						}
					}
				}
			}
		}

		let mut not_pure = HashSet::new();
		while let Some(funcname) = work_list.pop_back() {
			if not_pure.contains(&funcname) {
				continue;
			}
			not_pure.insert(funcname.clone());
			if let Some(item) = inverse_call_relation.get(&funcname) {
				for func in item {
					if not_pure.contains(func) {
						continue;
					}
					work_list.push_back(func.clone());
				}
			}
		}

		for func in program.funcs.iter_mut() {
			for block in &func.cfg.blocks {
				for instr in &block.borrow().instrs {
					if let Some(item) = instr.external_resorce() {
						// func.external_resorce.insert(item);
						// match &item {
						// 	utils::purity::ExternalResource::Call(callee) => {
						// 		if not_pure.contains(callee) {
						// 			func.external_resorce.insert(item);
						// 		}
						// 	}
						// 	_ => {}
						// }
						if let utils::purity::ExternalResource::Call(callee) = &item {
							if not_pure.contains(callee) {
								func.external_resorce.insert(item);
							}
						}
					}
				}
			}
		}

		Ok(false)
	}
}
