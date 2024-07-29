use super::LocalizeVariable;
use std::collections::{HashMap, HashSet};

use crate::RrvmOptimizer;
use llvm::{
	from_globalvar, mv_instr, LlvmInstr, LlvmInstrVariant, LlvmTemp,
	LlvmTempManager, LoadInstr, PhiInstr, Value, VarType,
};
use rrvm::{
	func::Entrance,
	program::{LlvmFunc, LlvmProgram},
	LlvmNode,
};
use utils::{errors::Result, GlobalVar, ValueItem::Zero};

fn move_local_array(program: &mut LlvmProgram) -> Result<bool> {
	let changed = false;
	let mut new_global = vec![];
	program
		.funcs
		.iter_mut()
		.filter(|func| func.entrance == Entrance::Single)
		.for_each(|func| {
			let name = func.name.clone();
			let new_name =
				|t: &LlvmTemp| format!("__optimized_local_array_{}_{}", &name, t.name);
			for block in &func.cfg.blocks {
				let mut new_instrs = vec![];
				let mut unkill_size = 0;
				std::mem::take(&mut block.borrow_mut().instrs).into_iter().for_each(
					|instr| {
						new_instrs.push(match instr.get_variant() {
							LlvmInstrVariant::AllocInstr(i) => {
								if let Value::Int(size) = i.length {
									if size > 4 {
										new_global.push(GlobalVar {
											ident: new_name(&i.target),
											data: vec![Zero(size as usize)],
											is_array: true,
											is_float: i.var_type.is_float(),
										});
										unkill_size += size;
										Box::new(LoadInstr {
											target: i.target.clone(),
											var_type: i.var_type,
											addr: Value::Temp(LlvmTemp {
												name: new_name(&i.target),
												is_global: true,
												var_type: i.var_type,
											}),
										})
									} else {
										instr
									}
								} else {
									instr
								}
							}
							_ => instr,
						})
					},
				);

				block.borrow_mut().instrs = new_instrs;
				block.borrow_mut().kill_size -= unkill_size;
			}
		});

	program.global_vars.append(&mut new_global);

	Ok(changed)
}

fn solve_func(func: &mut LlvmFunc, var: &GlobalVar, mgr: &mut LlvmTempManager) {
	let value = from_globalvar(var).unwrap();

	let var_type = if var.is_float {
		VarType::F32
	} else {
		VarType::I32
	};
	let mut get_tmp = || mgr.new_temp(var_type, false);
	let target_temp = LlvmTemp {
		name: var.ident.clone(),
		is_global: true,
		var_type,
	}
	.into();

	let target_load = |instr: &LlvmInstr| {
		let mut result = None;

		if let LlvmInstrVariant::LoadInstr(i) = instr.get_variant() {
			if i.addr == target_temp {
				result = Some(i.target.clone())
			}
		}

		result
	};

	let target_store = |instr: &LlvmInstr, target: &HashSet<LlvmTemp>| {
		let mut result = None;

		if let LlvmInstrVariant::StoreInstr(i) = instr.get_variant() {
			if target.contains(&i.addr.clone().get_temp().unwrap()) {
				result = i.value.clone().into()
			}
		}

		result
	};

	let mut last_store: HashMap<i32, Value> = HashMap::new();
	let mut beginning_phi: HashMap<i32, LlvmTemp> = HashMap::new();

	let entry_id = func.cfg.get_entry().borrow().id;

	for block in &mut func.cfg.blocks {
		let node: &mut LlvmNode = block;
		let id = node.borrow().id;
		let loads = block
			.borrow()
			.instrs
			.iter()
			.filter_map(target_load)
			.collect::<HashSet<_>>();
		// dbg!(&loads);
		for item in block
			.borrow()
			.instrs
			.iter()
			.rev()
			.filter_map(|instr| target_store(instr, &loads))
			.take(1)
		{
			last_store.insert(id, item);
		}
		if id != entry_id {
			beginning_phi.insert(id, get_tmp());
		}
	}

	if let std::collections::hash_map::Entry::Vacant(e) =
		last_store.entry(entry_id)
	{
		let entry_target = get_tmp();
		func
			.cfg
			.get_entry()
			.borrow_mut()
			.instrs
			.push(Box::new(mv_instr(value.clone(), entry_target.clone())));

		e.insert(entry_target.into());
	}

	// dbg!(&last_store);

	let get_last = |id: i32| {
		last_store
			.get(&id)
			.cloned()
			.or_else(|| beginning_phi.get(&id).map(|t| Value::Temp(t.clone())))
			.unwrap()
	};

	for block in &mut func.cfg.blocks {
		let id = block.borrow().id;

		let mut last_value = if id == entry_id {
			value.clone()
		} else {
			Value::Temp(beginning_phi.get(&id).unwrap().clone())
		};

		let src = block
			.borrow()
			.prev
			.iter()
			.map(|prev| {
				let prev_id = prev.borrow().id;
				(get_last(prev_id).clone(), prev.borrow().label())
			})
			.collect();

		if let Some(phitarget) = last_value.get_temp_ref() {
			let instr = PhiInstr {
				target: phitarget.clone(),
				var_type,
				source: src,
			};
			// println!("{}", &instr);
			block.borrow_mut().phi_instrs.push(instr);
		}

		let mut new_instrs: Vec<LlvmInstr> = vec![];

		let loads = block
			.borrow()
			.instrs
			.iter()
			.filter_map(target_load)
			.collect::<HashSet<_>>();

		for instr in std::mem::take(&mut block.borrow_mut().instrs).into_iter() {
			match instr.get_variant() {
				LlvmInstrVariant::StoreInstr(store) => {
					if loads.contains(store.addr.get_temp_ref().unwrap()) {
						last_value = store.to_owned().value;
					} else {
						new_instrs.push(instr)
					}
				}
				LlvmInstrVariant::LoadInstr(load) => {
					if loads.contains(&load.target) {
						// Load address of global variable
					} else if loads.contains(load.addr.get_temp_ref().unwrap()) {
						// load from memory
						new_instrs
							.push(Box::new(mv_instr(last_value.clone(), load.target.clone())))
					} else {
						new_instrs.push(instr)
					}
				}
				_ => new_instrs.push(instr),
			}
		}

		block.borrow_mut().instrs = new_instrs;
	}
}

fn move_global_scalar(program: &mut LlvmProgram) -> Result<bool> {
	let mut not_appliable = HashSet::new();
	program
		.funcs
		.iter()
		.filter(|func| {
			func.name.as_str() != "main" && func.entrance != Entrance::Never
		})
		.for_each(|func| {
			func.cfg.blocks.iter().for_each(|block| {
				block.borrow().instrs.iter().for_each(|instr| {
					if let LlvmInstrVariant::LoadInstr(instr) = instr.get_variant() {
						if let Value::Temp(t) = &instr.addr {
							if t.is_global {
								not_appliable.insert(t.name.clone());
							}
						}
					}
				})
			})
		});

	let mut killed_global = vec![];

	std::mem::take(&mut program.global_vars).into_iter().for_each(|var| {
		if var.is_array || not_appliable.contains(&var.ident) {
			program.global_vars.push(var);
		} else {
			killed_global.push(var);
		}
	});

	if let Some(main_func) =
		program.funcs.iter_mut().find(|func| func.name.as_str() == "main")
	{
		killed_global
			.into_iter()
			.map(|var| solve_func(main_func, &var, &mut program.temp_mgr))
			.count();
	}

	Ok(false)
}

impl RrvmOptimizer for LocalizeVariable {
	fn new() -> Self {
		LocalizeVariable {}
	}

	fn apply(self, program: &mut LlvmProgram) -> Result<bool> {
		program.analysis();
		let move_local_array = move_local_array(program)?;
		let move_global_scalar = move_global_scalar(program)?;
		program.analysis();
		Ok(move_local_array | move_global_scalar)
	}
}
