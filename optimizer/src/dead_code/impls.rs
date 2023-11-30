use std::collections::HashSet;

use super::RemoveDeadCode;
use crate::RrvmOptimizer;
use rrvm::{program::LlvmProgram, LlvmCFG};
use utils::errors::Result;

impl RrvmOptimizer for RemoveDeadCode {
	fn new() -> Self {
		Self {}
	}
	fn apply(self, program: &mut LlvmProgram) -> Result<()> {
		fn solve(cfg: &mut LlvmCFG) {
			let mut visited = HashSet::new();
			// let cfg = &mut func.cfg;
			let mut stack = vec![cfg.get_entry()];
			while let Some(u) = stack.pop() {
				let id = u.borrow().id;
				visited.insert(id);
				//TODO: skip empty block
				// remove unreachable branch
				let new_jump = u.borrow().jump_instr.as_ref().unwrap().new_jump();
				if let Some(instr) = new_jump {
					let label = &instr.target;
					u.borrow_mut().succ.retain(|v| v.borrow().label() == *label);
					if u.borrow().succ.len() == 2 {
						u.borrow_mut().succ.pop();
					}
					u.borrow_mut().set_jump(Some(Box::new(instr)));
				}
				// merge adjust block
				while u.borrow().single_succ() {
					let v = u.borrow().get_succ();
					if u.borrow().id != v.borrow().id
						&& v.borrow().single_prev()
						&& v.borrow().no_phi()
					{
						u.borrow_mut().instrs.append(&mut v.borrow_mut().instrs);
						let label = v.borrow().label();
						for succ in v.borrow().succ.iter() {
							succ.borrow_mut().replace_prev(&label, u.clone())
						}
						u.borrow_mut().succ.clear();
						u.borrow_mut().succ.append(&mut v.borrow_mut().succ);
						let instr = v.borrow_mut().jump_instr.take();
						u.borrow_mut().set_jump(instr);
					} else {
						break;
					}
				}
				// dfs
				for v in u.borrow().succ.iter() {
					if visited.get(&v.borrow().id).is_none() {
						stack.push(v.clone())
					}
				}
			}
			cfg.blocks.retain(|v| {
				visited.get(&v.borrow().id).is_some() || {
					v.borrow_mut().clear();
					false
				}
			});
			// solve data flow
			cfg.blocks.iter().for_each(|v| v.borrow_mut().prev.clear());
			cfg.blocks.iter().for_each(|u| {
				u.borrow().succ.iter().for_each(|v| v.borrow_mut().prev.push(u.clone()))
			});
			for block in cfg.blocks.iter() {
				let labels: HashSet<_> =
					block.borrow().prev.iter().map(|v| v.borrow().label()).collect();
				for instr in block.borrow_mut().phi_instrs.iter_mut() {
					instr.source.retain(|(_, label)| labels.get(label).is_some())
				}
			}
		}
		for func in program.funcs.iter_mut() {
			loop {
				let size = func.cfg.size();
				solve(&mut func.cfg);
				if func.cfg.size() == size {
					break;
				}
			}
		}
		Ok(())
	}
}
