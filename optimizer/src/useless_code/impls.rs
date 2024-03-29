use std::collections::{HashMap, HashSet, VecDeque};

use super::RemoveUselessCode;
use crate::RrvmOptimizer;
use llvm::{JumpInstr, LlvmTemp};
use rrvm::{dominator::*, program::LlvmProgram, LlvmCFG, LlvmNode};
use utils::{errors::Result, UseTemp};

impl RrvmOptimizer for RemoveUselessCode {
	fn new() -> Self {
		Self {}
	}
	fn apply(self, program: &mut LlvmProgram) -> Result<bool> {
		fn solve(cfg: &mut LlvmCFG) -> bool {
			let mut flag: bool = false;

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

			// LlvmTemp -> Instruction, id of the Basicblock which contains the instruction
			// instruction here is represented by its index in the basicblock
			let mut temp_graph: HashMap<LlvmTemp, HashSet<(LlvmTemp, i32)>> =
				HashMap::new();
			let mut worklist: VecDeque<LlvmTemp> = VecDeque::new();
			let mut visited: HashSet<LlvmTemp> = HashSet::new();
			let mut visited_block: HashSet<i32> = HashSet::new();
			let mut id_to_virtual_temp: HashMap<i32, LlvmTemp> = HashMap::new();

			for block in cfg.blocks.iter() {
				let block = block.borrow();
				let id = block.id;
				let virtual_temp = LlvmTemp {
					name: format!("virtual_temp_{}", id),
					is_global: false,
					var_type: llvm::VarType::Void,
				};
				id_to_virtual_temp.insert(id, virtual_temp.clone());
			}

			let mut insert_worklist = |t: &LlvmTemp, id: i32| {
				if !visited.contains(t) {
					visited.insert(t.clone());
					worklist.push_back(t.clone());
				}

				// virtual_temp 用来表示一个基本块是否有用，它将与这个基本块内所有定义的TEMP连边，但是基本块内可能没有定义temp,
				// 所以这里在发现这个块有用时，直接将 virtual temp 插入
				if !visited_block.contains(&id) {
					visited_block.insert(id);
					let v_temp = id_to_virtual_temp[&id].clone();
					if !visited.contains(&v_temp) {
						visited.insert(id_to_virtual_temp[&id].clone());
						worklist.push_back(id_to_virtual_temp[&id].clone())
					}
				}
			};
			let mut add_edge = |u: &LlvmTemp, v: &LlvmTemp, id: i32| {
				temp_graph.entry(u.clone()).or_default().insert((v.clone(), id));
			};
			for block in cfg.blocks.iter() {
				let block = block.borrow();
				let id = block.id;
				for instr in block.instrs.iter() {
					if instr.has_sideeffect() {
						instr.get_write().iter().for_each(|v| insert_worklist(v, id));
						instr.get_read().iter().for_each(|v| insert_worklist(v, id));
					}
				}
				let virtual_temp = id_to_virtual_temp[&id].clone();
				if let Some(jump) = block.jump_instr.as_ref() {
					if jump.is_ret() {
						jump.get_read().iter().for_each(|v| insert_worklist(v, id));
						insert_worklist(&virtual_temp, id);
					}
				}
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
						for prev in block.prev.iter() {
							let prev_id = prev.borrow().id;
							add_edge(&u, &id_to_virtual_temp[&prev_id], prev_id);
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
						}

						if !visited_block.contains(id) {
							visited_block.insert(*id);
							let v_temp = id_to_virtual_temp[id].clone();
							if !visited.contains(&v_temp) {
								visited.insert(id_to_virtual_temp[id].clone());
								worklist.push_back(id_to_virtual_temp[id].clone())
							}
						}
					}
				}
			}

			// Sweep. Clear the useless code
			for block in cfg.blocks.iter_mut() {
				let mut block = block.borrow_mut();
				block.instrs.retain(|instr| {
					instr.get_write().map_or(true, |v| visited.contains(&v)) || {
						flag = true;
						false
					}
				});
				block.phi_instrs.retain(|instr| {
					instr.get_write().map_or(true, |v| visited.contains(&v)) || {
						flag = true;
						false
					}
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
						new_target = Some(domi.borrow().label());
						block.succ.clear();
						block.succ.push(domi.clone());
					}
				}
				if let Some(t) = new_target {
					flag = true;
					block.jump_instr = Some(Box::new(JumpInstr { target: t }));
				}
			}
			cfg.resolve_prev();
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
