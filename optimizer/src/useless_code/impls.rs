use std::collections::{HashMap, HashSet};

use super::RemoveUselessCode;
use crate::RrvmOptimizer;
use llvm::Temp;
use rrvm::{program::LlvmProgram, LlvmCFG};
use utils::{errors::Result, UseTemp};

impl RrvmOptimizer for RemoveUselessCode {
	fn new() -> Self {
		Self {}
	}
	fn apply(self, program: &mut LlvmProgram) -> Result<()> {
		fn solve(cfg: &mut LlvmCFG) {
			let mut effect_in = HashMap::<i32, HashSet<Temp>>::new();
			let mut effect_out = HashMap::<i32, HashSet<Temp>>::new();
			loop {
				let mut changed = false;
				for u in cfg.blocks.iter().rev() {
					let mut new_effect_out = HashSet::new();

					new_effect_out.extend(
						u.borrow()
							.jump_instr
							.as_ref()
							.map_or(Vec::new(), |ret| ret.get_read()),
					);
					for v in u.borrow().succ.iter() {
						new_effect_out.extend(
							effect_in.get(&v.borrow().id).cloned().unwrap_or(HashSet::new()),
						);
					}

					let mut new_effect_in = new_effect_out.clone();
					for instr in u.borrow().instrs.iter().rev() {
						if instr
							.get_write()
							.map_or(false, |v| new_effect_in.remove(&v) || v.is_global)
							|| instr.is_store()
						{
							new_effect_in.extend(instr.get_read());
						}
					}
					for instr in u.borrow().phi_instrs.iter() {
						if instr
							.get_write()
							.map_or(false, |v| new_effect_in.remove(&v) || v.is_global)
						{
							new_effect_in.extend(instr.get_read());
						}
					}
					// TODO: can we not clone here?
					if new_effect_in
						!= effect_in.get(&u.borrow().id).cloned().unwrap_or(HashSet::new())
						|| new_effect_out
							!= effect_out
								.get(&u.borrow().id)
								.cloned()
								.unwrap_or(HashSet::new())
					{
						effect_in.insert(u.borrow().id, new_effect_in);
						effect_out.insert(u.borrow().id, new_effect_out);
						changed = true;
					}
				}
				if !changed {
					break;
				}
			}
			// println!("effect_in {:?}", effect_in);
			// println!("effect_out {:?}", effect_out);
			for u in cfg.blocks.iter().rev() {
				let mut u_effect_out =
					effect_out.get(&u.borrow().id).cloned().unwrap_or(HashSet::new());

				let mut new_instr = Vec::new();
				let mut new_phi_instr = Vec::new();
				for instr in u.borrow().instrs.iter().rev() {
					if instr
						.get_write()
						.map_or(false, |v| u_effect_out.remove(&v) || v.is_global)
						|| instr.is_store()
					{
						u_effect_out.extend(instr.get_read());
						new_instr.push(instr.clone_box());
					}
				}
				for instr in u.borrow().phi_instrs.iter() {
					if instr
						.get_write()
						.map_or(false, |v| u_effect_out.remove(&v) || v.is_global)
					{
						u_effect_out.extend(instr.get_read());
						new_phi_instr.push(instr.clone());
					}
				}
				new_instr.reverse();
				u.borrow_mut().instrs = new_instr;
				u.borrow_mut().phi_instrs = new_phi_instr;
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
