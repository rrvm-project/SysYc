use std::collections::{HashMap, HashSet, VecDeque};

use super::RemoveUselessCode;
use crate::RrvmOptimizer;
use llvm::{JumpInstr, Temp};
use rrvm::{
	dominator::{
		dominator_frontier::compute_dominator_frontier, naive::compute_dominator,
	},
	program::LlvmProgram,
	LlvmCFG, LlvmNode,
};
use utils::{errors::Result, UseTemp};

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
				&dominates_directly,
				&dominator,
				&mut dominator_frontier,
			);

			// Temp -> Instruction, id of the Basicblock which contains the instruction
			// instruction here is represented by its index in the basicblock
			let mut temp_graph: HashMap<Temp, HashSet<(Temp, i32)>> = HashMap::new();
			let mut worklist: VecDeque<Temp> = VecDeque::new();
			let mut visited: HashSet<Temp> = HashSet::new();
			let mut visited_block: HashSet<i32> = HashSet::new();
			let mut insert_worklist = |t: &Temp, id: i32| {
				if !visited.contains(t) {
					visited.insert(t.clone());
					worklist.push_back(t.clone());
					visited_block.insert(id);
				}
			};
			let mut add_edge = |u: &Temp, v: &Temp, id: i32| {
				temp_graph.entry(u.clone()).or_default().insert((v.clone(), id));
			};
			for block in cfg.blocks.iter() {
				let block = block.borrow();
				let id = block.id;
				for instr in block.instrs.iter() {
					if instr.has_sideeffect() {
						instr.get_write().iter().for_each(|v| insert_worklist(v, id));
					}
				}
				if let Some(jump) = block.jump_instr.as_ref() {
					if jump.is_ret() {
						jump.get_read().iter().for_each(|v| insert_worklist(v, id));
					}
				}
				let virtual_temp = Temp {
					name: format!("virtual_temp_{}", id),
					is_global: false,
					var_type: llvm::VarType::Void,
				};
				for instr in block.instrs.iter() {
					if let Some(u) = instr.get_write() {
						for v in instr.get_read() {
							add_edge(&u, &v, id);
						}
						add_edge(&u, &virtual_temp, id);
					}
				}
				for instr in block.phi_instrs.iter() {
					if let Some(u) = instr.get_write() {
						for v in instr.get_read() {
							add_edge(&u, &v, id);
						}
						add_edge(&u, &virtual_temp, id);
					}
				}
				for bb in dominator_frontier.get(&id).iter().flat_map(|v| v.iter()) {
					let bb_id = bb.borrow().id;
					if let Some(jump) = bb.borrow().jump_instr.as_ref() {
						jump
							.get_read()
							.iter()
							.for_each(|v| add_edge(&virtual_temp, v, bb_id));
					}
				}
			}

			while let Some(u) = worklist.pop_front() {
				if let Some(edges) = temp_graph.get(&u) {
					for (v, id) in edges.iter() {
						if !visited.contains(v) {
							visited.insert(v.clone());
							worklist.push_back(v.clone());
							visited_block.insert(*id);
						}
					}
				}
			}

			// Sweep. Clear the useless code
			for block in cfg.blocks.iter_mut() {
				let mut block = block.borrow_mut();

				block.instrs.retain(|instr| {
					if let Some(u) = instr.get_write() {
						if visited.contains(&u) {
							return true;
						}
					}
					false
				});

				block.phi_instrs.retain(|instr| {
					if let Some(u) = instr.get_write() {
						if visited.contains(&u) {
							return true;
						}
					}
					false
				});
			}

			for block in cfg.blocks.iter_mut() {
				let block_id = block.borrow().id;
				let mut block = block.borrow_mut();

				let mut new_target = None;

				if let Some(jump) = block.jump_instr.as_ref() {
					if jump.is_jump_cond() && !visited_block.contains(&block_id) {
						let mut domi = dominator.get(&block_id).unwrap();
						while domi.borrow().jump_instr.as_ref().unwrap().is_jump_cond()
							&& !visited_block.contains(&domi.borrow().id)
						{
							domi = dominator.get(&domi.borrow().id).unwrap();
						}
						new_target = Some(domi.borrow().label())
					}
				}
				if new_target.is_some() {
					block.jump_instr = Some(Box::new(JumpInstr {
						_attrs: HashMap::new(),
						target: new_target.unwrap(),
					}));
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
