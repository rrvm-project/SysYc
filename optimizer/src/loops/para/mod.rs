mod check_loop;
mod make_parallel;
mod pointer_tracer;

use std::collections::{HashMap, HashSet};

use check_loop::check_ok;
use llvm::{LlvmInstrVariant, LlvmTemp, LlvmTempManager, Value};
use make_parallel::make_parallel;
use pointer_tracer::PointerTracer;
use rrvm::{
	cfg::Node,
	dominator::{DomTree, LlvmDomTree},
	program::{LlvmFunc, LlvmProgram},
	rrvm_loop::LoopPtr,
};

use crate::metadata::MetaData;

use super::{
	indvar::IndVar, indvar_type::IndVarType, loop_data::LoopData,
	loopinfo::LoopInfo, HandleLoops,
};
use utils::{InstrTrait, Result, TempTrait};

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

struct DominatorDFS<T, U>
where
	T: InstrTrait<U>,
	U: TempTrait,
{
	dom_tree: DomTree<T, U>,
	stack: Vec<Node<T, U>>,
}

impl<T, U> Iterator for DominatorDFS<T, U>
where
	T: InstrTrait<U>,
	U: TempTrait,
{
	type Item = Node<T, U>;

	fn next(&mut self) -> Option<Self::Item> {
		if let Some(current) = self.stack.pop() {
			let current_id = current.borrow().id;
			self.stack.extend(self.dom_tree.get_children(current_id).iter().cloned());
			current.into()
		} else {
			None
		}
	}
}

impl<T, U> DominatorDFS<T, U>
where
	T: InstrTrait<U>,
	U: TempTrait,
{
	pub fn new(dom_tree: DomTree<T, U>, entry: Node<T, U>) -> Self {
		Self {
			dom_tree,
			stack: vec![entry],
		}
	}
}

fn handle_function(
	func: &mut LlvmFunc,
	loop_data: LoopData,
	mgr: &mut LlvmTempManager,
) -> LoopData {
	if func.name != "main" {
		//似乎只在main中工作
		return loop_data;
	}
	let (mut loop_map, root_loop, mut loop_infos, mut indvars) = (
		loop_data.loop_map,
		loop_data.root_loop,
		loop_data.loop_infos,
		loop_data.indvars,
	);
	//loop map: 所有的loop 都有
	//loop info: 如果没有一定不能并行

	let mut ok_loop_id: HashSet<i32> = HashSet::new();

	let mut ptr_set: pointer_tracer::PointerTracer = PointerTracer::new();
	let mut indvar_ptr_set: pointer_tracer::PointerTracer = PointerTracer::new();

	fn get_i32(value: &Value) -> Option<i32> {
		match value {
			Value::Int(i) => Some(*i),
			_ => None,
		}
	}

	for candidate_loop in &root_loop.borrow().subloops {
		for block in
			candidate_loop.borrow().blocks_without_subloops(&func.cfg, &loop_map)
		{
			for targets in block.borrow().phi_instrs.iter().map(|instr| &instr.target)
			{
				if targets.var_type.is_ptr() {
					indvar_ptr_set.create(targets);
				}
			}

			for gep in block.borrow().instrs.iter().filter_map(|instr| {
				match instr.get_variant() {
					LlvmInstrVariant::GEPInstr(gep) => Some(gep),
					_ => None,
				}
			}) {
				if let Some(target_ind) = indvars.get(&gep.target) {
					if target_ind.get_type() == IndVarType::Ordinary {
						if let (Some(step), offset) = (
							get_i32(target_ind.step.first().unwrap()),
							get_i32(&gep.offset),
						) {
							if step == 0 {
								//HACK: 取消注释这行可能引起错误
								//indvar_ptr_set.create(&gep.target);
							} else if offset.is_some_and(|offset| offset < step) {
								indvar_ptr_set
									.link(&gep.target, gep.addr.unwrap_temp_ref().unwrap());
							} else {
								indvar_ptr_set.create(&gep.target);
							}
						} else {
							indvar_ptr_set.create(&gep.target);
						}
					}
				}
			}
		}
	}

	// 假定传入参数的不同的指针指向不重叠的内存

	for param in &func.params {
		if let Some(t) = get_temp_ref(param) {
			if t.var_type.is_ptr() {
				ptr_set.create(t);
			}
		}
	}

	let dom_tree = LlvmDomTree::new(&func.cfg, false);

	for block in DominatorDFS::new(dom_tree, func.cfg.get_entry()) {
		for instr in &block.borrow().phi_instrs {
			if let Some(to_link) = instr
				.source
				.iter()
				.find(|(v, _)| v.unwrap_temp_ref().is_some_and(|t| ptr_set.know(t)))
				.and_then(|(v, _)| v.unwrap_temp_ref())
			{
				ptr_set.link(&instr.target, to_link);
				indvar_ptr_set.link(&instr.target, to_link);
			}
		}

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
						indvar_ptr_set.link(&i.target, t);
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

	// dbg!(&indvar_ptr_set, &ptr_set);

	check_ok(
		root_loop.clone(),
		&mut ptr_set,
		&mut indvar_ptr_set,
		&mut ok_loop_id,
		&func.cfg,
		&loop_map,
	);

	// dbg!(&ok_loop_id);

	parallel_loop(
		root_loop.clone(),
		&ok_loop_id,
		&mut loop_map,
		&mut loop_infos,
		mgr,
		&mut indvars,
	);

	let temp_graph = LoopData::build_graph(func);
	let def_map = LoopData::build_def_map(func);

	LoopData {
		temp_graph,
		loop_map,
		def_map,
		root_loop,
		loop_infos,
		indvars,
	}
}

fn last_check(info: LoopInfo, indvars: &mut HashMap<LlvmTemp, IndVar>) -> bool {
	let header = info.header.clone();
	let exit = info.single_exit.clone();

	if !matches!(
		info.comp_op,
		llvm::CompOp::SGT
			| llvm::CompOp::SGE
			| llvm::CompOp::SLT
			| llvm::CompOp::SLE
	) {
		return false;
	}

	if header.borrow().phi_instrs.is_empty() {
		return false;
	}

	for item in header.borrow().phi_instrs.iter() {
		if indvars.get(&item.target).is_some_and(|indvar| {
			matches!(
				indvar.get_type(),
				crate::loops::indvar_type::IndVarType::Ordinary // | crate::loops::indvar_type::IndVarType::OrdinaryZFP
			)
		}) {
			continue;
		}
		return false;
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

	// dbg!(header.borrow().phi_instrs.len());
	jump_ok
}

fn parallel_loop(
	current: LoopPtr,
	ok: &HashSet<i32>,
	loop_map: &mut HashMap<i32, LoopPtr>,
	loop_info: &mut HashMap<i32, LoopInfo>,
	mgr: &mut LlvmTempManager,
	indvars: &mut HashMap<LlvmTemp, IndVar>,
) {
	let current_id = current.borrow().id;

	let mut operate_on_this = ok.contains(&current_id);

	let is_single_layer = current.borrow().subloops.is_empty();

	let mut operated = false;

	if let Some(info) = loop_info.get_mut(&current_id) {
		if let (Value::Int(start), Value::Int(step), Value::Int(end)) =
			(&info.begin, &info.step, &info.end)
		{
			if is_single_layer && ((end - start) / step) < 4096 {
				operate_on_this = false;
			}
		}

		let preheader = info.preheader.clone();

		if operate_on_this && last_check(info.clone(), indvars) {
			let pre_id = preheader.borrow().id;
			if loop_map.get(&pre_id).cloned().is_some() {
				let (new_start, new_end, new_index) =
					make_parallel(info.clone(), mgr, indvars);

				indvars.insert(
					new_index,
					IndVar {
						base: 0.into(),
						scale: 1.into(),
						step: vec![1.into()],
						zfp: None,
					},
				);

				info.begin = new_start;
				info.end = new_end;

				operated = true;
			}
		}
	}

	if operated {
		eprintln!(
			"para {} B{}",
			current.borrow().id,
			current.borrow().header.borrow().id
		);
	}

	if operated || current_id != 1 {
		return;
	}

	for sub in current.borrow().subloops.iter().cloned() {
		parallel_loop(sub, ok, loop_map, loop_info, mgr, indvars);
	}
}
