use std::{cell::RefCell, collections::HashSet, rc::Rc};

use super::RemoveUnreachCode;
use crate::RrvmOptimizer;
use rrvm::{cfg::BasicBlock, program::LlvmProgram};
use utils::errors::Result;

impl RrvmOptimizer for RemoveUnreachCode {
	fn new() -> Self {
		Self {}
	}
	fn apply(self, program: &mut LlvmProgram) -> Result<bool> {
		Ok(program.funcs.iter_mut().fold(false, |last, func| {
			let size = func.cfg.size();
			let mut visited = HashSet::new();
			let cfg = &mut func.cfg;
			let mut stack = vec![cfg.get_entry()];
			while let Some(u) = stack.pop() {
				let id = u.borrow().id;
				visited.insert(id);
				// dfs
				for v in u.borrow().succ.iter() {
					if !visited.contains(&v.borrow().id) {
						stack.push(v.clone())
					}
				}
			}
			cfg.blocks.retain(|v| {
				visited.contains(&v.borrow().id) || {
					v.borrow_mut().clear();
					false
				}
			});

			// 检查是否为单一出口，如不是，则补一个虚拟出口
			let mut block_has_ret = Vec::new();
			for bb in cfg.blocks.iter() {
				if bb.borrow().jump_instr.is_some()
					&& bb.borrow().jump_instr.as_ref().unwrap().is_ret()
				{
					block_has_ret.push(bb.clone());
				}
			}
			let exit;
			if block_has_ret.len() == 1 {
				exit = block_has_ret[0].clone();
			} else {
				exit = Rc::new(RefCell::new(BasicBlock::new(-1, 0.0)));
				block_has_ret.iter().for_each(|bb| {
					bb.borrow_mut().succ.push(exit.clone());
					exit.borrow_mut().prev.push(bb.clone());
				});
			}

			// 复用前面的变量，先清空
			visited.clear();
			stack.clear();
			stack.push(exit);

			// dfs
			while let Some(u) = stack.pop() {
				let id = u.borrow().id;
				visited.insert(id);
				for v in u.borrow().prev.iter() {
					if !visited.contains(&v.borrow().id) {
						stack.push(v.clone())
					}
				}
			}
			// 清除不可达基本块
			cfg.blocks.retain(|v| {
				visited.contains(&v.borrow().id) || {
					v.borrow_mut().clear();
					false
				}
			});
			
			// 清楚虚拟出口的残留影响
			block_has_ret.iter().for_each(|bb| {
				bb.borrow_mut().succ.clear();
			});

			last || size != cfg.blocks.len()
		}))
	}
}
