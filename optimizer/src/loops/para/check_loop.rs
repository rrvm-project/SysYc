use std::collections::{HashMap, HashSet};

use rrvm::{rrvm_loop::LoopPtr, LlvmCFG};

use super::{get_temp_ref, pointer_tracer::PointerTracer};

pub fn check_ok(
	current_loop: LoopPtr,
	ptr_tracer: &mut PointerTracer,
	ok: &mut HashSet<u32>,
	cfg: &LlvmCFG,
	loop_map: &HashMap<i32, LoopPtr>,
) -> bool {
	let mut failed = false;
	for sub_loop in &current_loop.borrow().subloops {
		let (read, write) = ptr_tracer.clear();
		failed |= check_ok(sub_loop.clone(), ptr_tracer, ok, cfg, loop_map);
		failed |= ptr_tracer.merge(read, write);
	}
	if failed {
		return true;
	};

	for bb in &current_loop.borrow().blocks_without_subloops(cfg, loop_map) {
		for instr in bb.borrow().instrs.iter() {
			match instr.get_variant() {
				llvm::LlvmInstrVariant::AllocInstr(_)
				| llvm::LlvmInstrVariant::CallInstr(_) => {
					return true;
				}
				llvm::LlvmInstrVariant::StoreInstr(i) => {
					if ptr_tracer.write(get_temp_ref(&i.addr).unwrap()) {
						return true;
					}
				}
				llvm::LlvmInstrVariant::LoadInstr(i) => {
					let temp = get_temp_ref(&i.addr).unwrap();
					if temp.is_global {
						continue;
					}
					if ptr_tracer.read(temp) {
						return true;
					}
				}
				_ => {}
			}
		}
	}
	ok.insert(current_loop.borrow().id);
	failed
}
