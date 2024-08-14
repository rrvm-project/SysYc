mod check_loop;
mod make_parallel;
mod pointer_tracer;

use std::collections::{HashMap, HashSet};

use check_loop::check_ok;
use llvm::{LlvmTemp, LlvmTempManager, Value};
use make_parallel::make_parallel;
use pointer_tracer::PointerTracer;
use rrvm::{
	program::{LlvmFunc, LlvmProgram},
	rrvm_loop::LoopPtr,
	LlvmNode,
};

use crate::metadata::MetaData;

use super::{loop_data::LoopData, loopinfo::LoopInfo, HandleLoops};
use utils::Result;

impl HandleLoops {
	pub fn parallel(
		&mut self,
		program: &mut LlvmProgram,
		_metadata: &mut MetaData,
	) -> Result<bool> {
		for func in program.funcs.iter_mut() {
			if let Some(loop_data) = self.loopdatas.remove(&func.name) {
				self.loopdatas.insert(
					func.name.clone(),
					handle_function(func, loop_data, &mut program.temp_mgr),
				);
			}
		}
		Ok(false)
	}
}

fn get_temp_ref(value: &Value) -> Option<&LlvmTemp> {
	match value {
		Value::Temp(t) => Some(t),
		_ => None,
	}
}

fn handle_function(
	func: &mut LlvmFunc,
	loop_data: LoopData,
	mgr: &mut LlvmTempManager,
) -> LoopData {
	let (mut loop_map, root_loop, mut loop_infos) = (
		loop_data.loop_map,
		loop_data.root_loop,
		loop_data.loop_infos,
	);
	//loop map: 所有的loop 都有
	//loop info: 如果没有一定不能并行

	let mut ok_loop_id: HashSet<u32> = HashSet::new();

	let mut ptr_set: pointer_tracer::PointerTracer = PointerTracer::new();

	// 假定传入参数的不同的指针指向不重叠的内存

	for param in &func.params {
		if let Some(t) = get_temp_ref(param) {
			if t.var_type.is_ptr() {
				ptr_set.create(t);
			}
		}
	}

	for block in &func.cfg.blocks {
		for instr in &block.borrow().instrs {
			match instr.get_variant() {
				llvm::LlvmInstrVariant::AllocInstr(i) => {
					ptr_set.create(&i.target);
				}
				llvm::LlvmInstrVariant::StoreInstr(i) => {
					if let Some(t) = get_temp_ref(&i.addr) {
						ptr_set.get(t);
					}
				}
				llvm::LlvmInstrVariant::LoadInstr(i) => {
					if let Some(t) = get_temp_ref(&i.addr) {
						if t.is_global {
							ptr_set.name(t, &t.name);
						}
						if i.target.var_type.is_ptr() {
							ptr_set.link(&i.target, t);
						}
					}
				}
				llvm::LlvmInstrVariant::GEPInstr(i) => {
					if let Some(t) = get_temp_ref(&i.addr) {
						ptr_set.link(&i.target, t);
					}
				}
				llvm::LlvmInstrVariant::CallInstr(i) => {
					if i.target.var_type.is_ptr() {
						//since function that returns ptr can only be our function to fill zeros, which returns the ptr in same array
						for (t, value) in i.params.iter() {
							if t.is_ptr() {
								ptr_set.link(&i.target, get_temp_ref(value).unwrap());
							}
						}
					}
				}
				_ => {}
			}
		}
	}

	check_ok(
		root_loop.clone(),
		&mut ptr_set,
		&mut ok_loop_id,
		&func.cfg,
		&loop_map,
	);
	parallel_loop(
		root_loop.clone(),
		&ok_loop_id,
		&mut loop_map,
		&mut loop_infos,
		mgr,
		&mut func.total,
		&mut func.cfg.blocks,
	);

	let temp_graph = LoopData::build_graph(func);
	let def_map = LoopData::build_def_map(func);

	LoopData {
		temp_graph,
		loop_map,
		def_map,
		root_loop,
		loop_infos,
	}
}

fn last_check(info: LoopInfo) -> bool {
	let header = info.header.clone();
	let exit = info.single_exit.clone();
	let cmp = info.cmp.clone();

	if header.borrow().phi_instrs.len() != 1 {
		return false;
	}

	let loop_var = header.borrow().phi_instrs.first().unwrap().target.clone();

	let mut check_cmp_ok = false;

	for instr in header.borrow().instrs.iter() {

		if let llvm::LlvmInstrVariant::CompInstr(i) = instr.get_variant() {
			if i.target == cmp {
				match (i.op, &i.lhs, &i.rhs) {
					(llvm::CompOp::SLT, Value::Temp(lhs), _) if *lhs == loop_var => {
						check_cmp_ok = true;
						break;
					}
					(llvm::CompOp::SGT, _, Value::Temp(rhs)) if *rhs == loop_var => {
						check_cmp_ok = true;
						break;
					}
					_ => {}
				}
			}
		}
	}

	let jump_ok = match header.borrow().jump_instr.as_ref() {
		Some(jump) => match jump.get_variant() {
			llvm::LlvmInstrVariant::JumpCondInstr(cond) => {
				cond.target_false == exit.borrow().label()
			}
			_ => false,
		},
		None => false,
	};

	let phi_ok = header.borrow().phi_instrs.len() == 1;

	// dbg!(header.borrow().phi_instrs.len());

	check_cmp_ok && jump_ok && phi_ok
}

fn parallel_loop(
	current: LoopPtr,
	ok: &HashSet<u32>,
	loop_map: &mut HashMap<i32, LoopPtr>,
	loop_info: &mut HashMap<u32, LoopInfo>,
	mgr: &mut LlvmTempManager,
	bb_cnt: &mut i32,
	blocks: &mut Vec<LlvmNode>,
) {
	let current_id = current.borrow().id;
	let current_ura_id = current.borrow().ura_id;
	//不并行单层
	let operate_on_this =
		ok.contains(&current_id) && current_id + 1 < current_ura_id;

	let mut operated = false;

	if let Some(info) = loop_info.get_mut(&current_id) {
		let preheader = info.preheader.clone();

		let cmp_op = info.comp_op;

		if (cmp_op == llvm::CompOp::SLT || cmp_op == llvm::CompOp::SGT)
			&& operate_on_this
			&& last_check(info.clone())
		{
			let pre_id = preheader.borrow().id;
			if let Some(outer) = loop_map.get(&pre_id).cloned() {
				// dbg!(&header.borrow().id, &preheader.borrow().id, &exit.borrow().id, &cmp, &step, &begin, &end);

				let (new_bb, new_start, new_end) =
					make_parallel(info.clone(), mgr, bb_cnt, blocks);
				for item in new_bb {
					loop_map.insert(item.borrow().id, outer.clone());
				}
				info.begin = Value::Temp(new_start);
				info.end = Value::Temp(new_end);

				operated = true;
			}
		}
	}

	if operated {
		eprintln!(
			"para {} {} B{}",
			current.borrow().id,
			current.borrow().ura_id,
			current.borrow().header.borrow().id
		);
	}

	if operated || current_id != 1 {
		return;
	}

	for sub in current.borrow().subloops.iter().cloned() {
		parallel_loop(sub, ok, loop_map, loop_info, mgr, bb_cnt, blocks);
	}
}
