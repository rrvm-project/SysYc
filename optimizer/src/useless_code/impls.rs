use std::collections::{HashMap, VecDeque};

use super::RemoveUselessCode;
use crate::RrvmOptimizer;
use llvm::{
	llvminstrattr::{LlvmAttr, LlvmAttrs},
	JumpInstr, LlvmInstr, Temp,
};
use rrvm::{
	dominator::{
		dominator_frontier::compute_dominator_frontier, naive::compute_dominator,
	},
	program::LlvmProgram,
	LlvmCFG, LlvmNode,
};
use utils::{errors::Result, UseTemp};

const MARK: &str = "MARK";

impl RrvmOptimizer for RemoveUselessCode {
	fn new() -> Self {
		Self {}
	}
	fn apply(self, program: &mut LlvmProgram) -> Result<bool> {
		fn solve(cfg: &mut LlvmCFG) {
			let mut _flag: bool = false;

			let mut dominates: HashMap<i32, Vec<LlvmNode>> = HashMap::new();
			let mut dominates_directly: HashMap<i32, Vec<LlvmNode>> = HashMap::new();
			let mut dominator: HashMap<i32, LlvmNode> = HashMap::new();
			compute_dominator(
				cfg,
				true,
				&mut dominates,
				&mut dominates_directly,
				&mut dominator,
			);

			let mut dominator_frontier: HashMap<i32, Vec<LlvmNode>> = HashMap::new();
			compute_dominator_frontier(
				cfg,
				true,
				&dominates,
				&dominator,
				&mut dominator_frontier,
			);

			// Temp -> Instruction, Basicblock which contains the instruction
			let mut defs: HashMap<Temp, (LlvmInstr, LlvmNode)> = HashMap::new();
			let mut worklist = VecDeque::new();
			for block in cfg.blocks.iter() {
				for instr in block.borrow_mut().instrs.iter_mut() {
					instr.clear_attr(MARK);
					if instr.is_store()
						|| instr
							.get_read()
							.iter()
							.chain(instr.get_write().iter())
							.any(|t| t.is_global)
					{
						instr.set_attr(MARK, LlvmAttr::Mark);
						worklist.push_back((instr.clone_box(), block.clone()))
					}
					if let Some(t) = instr.get_write().clone() {
						if !t.is_global {
							defs.insert(t, (instr.clone_box(), block.clone()));
						}
					}
				}
				if let Some(instr) = block.borrow_mut().jump_instr.as_mut() {
					if instr.is_ret() || instr.is_direct_jump() {
						instr.set_attr(MARK, LlvmAttr::Mark);
						worklist.push_back((instr.clone_box(), block.clone()))
					}
				}
			}

			while let Some((instr, basicblock)) = worklist.pop_front() {
				instr.get_read().iter().for_each(|t| {
					if let Some((instr_inner, bb_inner)) = defs.get_mut(t) {
						if instr_inner.get_attr(MARK).is_none() {
							instr_inner.set_attr(MARK, LlvmAttr::Mark);
							worklist.push_back((instr_inner.clone_box(), bb_inner.clone()));
						}
					}
				});
				dominator_frontier
					.entry(basicblock.borrow().id)
					.or_default()
					.iter_mut()
					.for_each(|bb| {
						if let Some(jump) = bb.borrow_mut().jump_instr.as_mut() {
							if jump.get_attr(MARK).is_none() {
								jump.set_attr(MARK, LlvmAttr::Mark);
								worklist.push_back((jump.clone_box(), bb.clone()));
							}
						}
					})
			}

			// Sweep. Clear the useless code
			for block in cfg.blocks.iter_mut() {
				block
					.borrow_mut()
					.instrs
					.retain(|instr| instr.get_attr(MARK).is_some());
				block
					.borrow_mut()
					.phi_instrs
					.retain(|instr| instr.get_attr(MARK).is_some());
				if let Some(jump) = block.borrow_mut().jump_instr.as_ref() {
					if jump.get_attr(MARK).is_none() && jump.is_jump_cond() {
						let mut domi = dominator.get(&block.borrow().id).unwrap();
						while !(domi
							.borrow()
							.phi_instrs
							.iter()
							.any(|i| i.get_attr(MARK).is_some())
							|| domi
								.borrow()
								.instrs
								.iter()
								.chain(domi.borrow().jump_instr.iter())
								.any(|i| i.get_attr(MARK).is_some()))
						{
							domi = dominator.get(&domi.borrow().id).unwrap();
						}
						block.borrow_mut().jump_instr.replace(Box::new(JumpInstr {
							_attrs: HashMap::from([(MARK.to_string(), LlvmAttr::Mark)]),
							target: domi.borrow().label(),
						}));
					}
				}
			}
		}
		// fn solve(cfg: &mut LlvmCFG) {
		// 	let mut flag: bool = false;
		// 	let mut dominates: HashMap<i32, Vec<LlvmNode>> = HashMap::new();
		// 	let mut dominates_directly: HashMap<i32, Vec<LlvmNode>> = HashMap::new();
		// 	let mut dominator: HashMap<i32, LlvmNode> = HashMap::new();
		// 	compute_dominator(
		// 		cfg,
		// 		true,
		// 		&mut dominates,
		// 		&mut dominates_directly,
		// 		&mut dominator,
		// 	);
		// 	let mut effect_in = HashMap::<i32, HashSet<Temp>>::new();
		// 	let mut effect_out = HashMap::<i32, HashSet<Temp>>::new();
		// 	loop {
		// 		let mut changed = false;
		// 		for u in cfg.blocks.iter().rev() {
		// 			let mut has_effective_instr = false;

		// 			let mut new_effect_out = effect_out.get(&u.borrow().id).cloned().unwrap_or(HashSet::new());

		// 			if let Some(jump_instr) = u.borrow().jump_instr.as_ref() {
		// 				if jump_instr.is_ret() {
		// 					has_effective_instr = true;
		// 					new_effect_out.extend(jump_instr.get_read());
		// 				}
		// 				// 如果是无条件跳转（read为空）或有条件跳转且
		// 				else if jump_instr.get_read().is_empty() ||{

		// 				}
		// 			}

		// 			for v in u.borrow().succ.iter() {
		// 				new_effect_out.extend(
		// 					effect_in.get(&v.borrow().id).cloned().unwrap_or(HashSet::new()),
		// 				);
		// 			}

		// 			let mut new_effect_in = new_effect_out.clone();
		// 			for instr in u.borrow().instrs.iter().rev() {
		// 				if instr
		// 					.get_write()
		// 					.map_or(false, |v| new_effect_in.remove(&v) || v.is_global)
		// 					|| instr.is_store()
		// 				{
		// 					new_effect_in.extend(instr.get_read());
		// 				}
		// 			}
		// 			for instr in u.borrow().phi_instrs.iter() {
		// 				if instr
		// 					.get_write()
		// 					.map_or(false, |v| new_effect_in.remove(&v) || v.is_global)
		// 				{
		// 					new_effect_in.extend(instr.get_read());
		// 				}
		// 			}
		// 			// TODO: can we not clone here?
		// 			if new_effect_in
		// 				!= effect_in.get(&u.borrow().id).cloned().unwrap_or(HashSet::new())
		// 				|| new_effect_out
		// 					!= effect_out
		// 						.get(&u.borrow().id)
		// 						.cloned()
		// 						.unwrap_or(HashSet::new())
		// 			{
		// 				effect_in.insert(u.borrow().id, new_effect_in);
		// 				effect_out.insert(u.borrow().id, new_effect_out);
		// 				changed = true;
		// 			}
		// 		}
		// 		if !changed {
		// 			break;
		// 		}
		// 	}
		// 	// println!("effect_in {:?}", effect_in);
		// 	// println!("effect_out {:?}", effect_out);
		// 	for u in cfg.blocks.iter().rev() {
		// 		let mut u_effect_out =
		// 			effect_out.get(&u.borrow().id).cloned().unwrap_or(HashSet::new());

		// 		let mut new_instr = Vec::new();
		// 		let mut new_phi_instr = Vec::new();
		// 		for instr in u.borrow().instrs.iter().rev() {
		// 			if instr
		// 				.get_write()
		// 				.map_or(false, |v| u_effect_out.remove(&v) || v.is_global)
		// 				|| instr.is_store()
		// 			{
		// 				u_effect_out.extend(instr.get_read());
		// 				new_instr.push(instr.clone_box());
		// 			}
		// 		}
		// 		for instr in u.borrow().phi_instrs.iter() {
		// 			if instr
		// 				.get_write()
		// 				.map_or(false, |v| u_effect_out.remove(&v) || v.is_global)
		// 			{
		// 				u_effect_out.extend(instr.get_read());
		// 				new_phi_instr.push(instr.clone());
		// 			}
		// 		}
		// 		new_instr.reverse();
		// 		u.borrow_mut().instrs = new_instr;
		// 		u.borrow_mut().phi_instrs = new_phi_instr;
		// 	}
		// }
		Ok(program.funcs.iter_mut().fold(false, |last, func| {
			let mut cnt = 0;
			loop {
				let size = func.cfg.size();
				solve(&mut func.cfg);
				if func.cfg.size() == size {
					break;
				}
				cnt += 1;
			}
			cnt != 0 || last
		}))
	}
}
