use std::{cell::RefCell, collections::HashSet, rc::Rc};

use super::RemoveUnreachCode;
use crate::RrvmOptimizer;
use llvm::{LlvmInstrTrait, LlvmTemp};
use rrvm::{cfg::BasicBlock, program::LlvmProgram};
use utils::errors::Result;

fn clear_prev_succ(
	v: &Rc<RefCell<BasicBlock<Box<dyn LlvmInstrTrait>, LlvmTemp>>>,
) {
	let this = v.borrow().id;

	for prev in &v.borrow().prev {
		//try_borrow失败的情况下，这里的prev就是v, 在稍后整个块会被直接清理，不用管
		if let Ok(mut prev) = prev.try_borrow_mut() {
			prev.succ.retain(|block| {
				// 如果这里block是prev, borrow 会失败（能走到这里，prev不是要删除的v）。is_ok_and返回的是false, 整体是true
				!block.try_borrow().is_ok_and(|block| block.id == this)
			});
		}
	}

	for succ in &v.borrow().succ {
		//try_borrow失败的情况下，这里的succ就是v, 在稍后整个块会被直接清理，不用管
		if let Ok(mut succ) = succ.try_borrow_mut() {
			succ.prev.retain(|block| {
				// 如果这里block是succ, borrow 会失败（能走到这里，succ不是要删除的v）。is_ok_and返回的是false, 整体是true
				!block.try_borrow().is_ok_and(|block| block.id == this)
			});
		}
	}
}

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
					clear_prev_succ(v);
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
			let visited_entry = visited.clone();

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

			visited.retain(|id| visited_entry.contains(id)); // 从entry和exit都可达

			cfg.blocks.iter_mut().for_each(|block| {
				block.borrow_mut().prev.retain(|block| {
					!block.try_borrow().is_ok_and(|block| !visited.contains(&block.id))
				});
				block.borrow_mut().succ.retain(|block| {
					!block.try_borrow().is_ok_and(|block| !visited.contains(&block.id))
				});
			});

			// 清除不可达基本块
			cfg.blocks.retain(|v| {
				visited.contains(&v.borrow().id) || {
					v.borrow_mut().clear();
					false
				}
			});

			// 清除虚拟出口的残留影响
			block_has_ret.iter().for_each(|bb| {
				bb.borrow_mut().succ.clear();
			});

			last || size != cfg.blocks.len()
		}))
	}
}
