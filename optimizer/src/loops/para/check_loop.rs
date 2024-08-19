use std::collections::{HashMap, HashSet};

use rrvm::{rrvm_loop::LoopPtr, LlvmCFG};

use super::{get_temp_ref, pointer_tracer::PointerTracer};

pub fn check_ok(
	root_loop: LoopPtr,
	ptr_tracer: &mut PointerTracer,
	indvar_ptr_tracer: &mut PointerTracer,
	ok: &mut HashSet<i32>,
	cfg: &LlvmCFG,
	loop_map: &HashMap<i32, LoopPtr>,
) {
	'current_loop: for current in root_loop.borrow().subloops.iter() {
		let mut wrote: HashSet<u32> = HashSet::new();
		let mut access: HashMap<u32, u32> = HashMap::new();

		fn set(map: &mut HashMap<u32, u32>, array: u32, indvar: u32) -> bool {
			// return true when failed!
			map.insert(array, indvar).is_some_and(|old_indvar| old_indvar != indvar)
		}

		for bb in &current.borrow().blocks(cfg, loop_map) {
			//with subloop
			for instr in bb.borrow().instrs.iter() {
				match instr.get_variant() {
					llvm::LlvmInstrVariant::AllocInstr(_)
					| llvm::LlvmInstrVariant::CallInstr(_) => {
						continue 'current_loop;
					}
					llvm::LlvmInstrVariant::StoreInstr(i) => {
						let wrote_tmp = get_temp_ref(&i.addr).unwrap();
						if let Some(array) = ptr_tracer.look_up(wrote_tmp) {
							wrote.insert(array);
						} else {
							continue 'current_loop;
						}
					}
					_ => {}
				}
			}
		}

		for bb in &current.borrow().blocks(cfg, loop_map) {
			//with subloop
			for instr in bb.borrow().instrs.iter() {
				match instr.get_variant() {
					llvm::LlvmInstrVariant::StoreInstr(i) => {
						let wrote_tmp = get_temp_ref(&i.addr).unwrap();

						if let Some(array) = ptr_tracer.look_up(wrote_tmp) {
							if wrote.contains(&array) {
								if let Some(indvar) = indvar_ptr_tracer.look_up(wrote_tmp) {
									if set(&mut access, array, indvar) {
										continue 'current_loop;
									}
								} else {
									continue 'current_loop;
								}
							}
						} else {
							continue 'current_loop;
						}
					}
					llvm::LlvmInstrVariant::LoadInstr(i) => {
						let temp = get_temp_ref(&i.addr).unwrap();
						if temp.is_global {
							continue;
						}
						if let Some(array) = ptr_tracer.look_up(temp) {
							if wrote.contains(&array) {
								if let Some(indvar) = indvar_ptr_tracer.look_up(temp) {
									if set(&mut access, array, indvar) {
										continue 'current_loop;
									}
								} else {
									continue 'current_loop;
								}
							}
						} else {
							continue 'current_loop;
						}
					}
					_ => {}
				}
			}
		}
		ok.insert(current.borrow().id);
	}
}
