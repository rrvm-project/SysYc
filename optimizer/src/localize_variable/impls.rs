use super::LocalizeVariable;
use std::collections::HashSet;

use crate::RrvmOptimizer;
use llvm::{LlvmInstrVariant, LlvmTemp, LoadInstr, Value};
use rrvm::{func::Entrance, program::LlvmProgram};
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

impl RrvmOptimizer for LocalizeVariable {
	fn new() -> Self {
		LocalizeVariable {}
	}

	fn apply(self, program: &mut LlvmProgram) -> Result<bool> {
		let move_local = move_local_array(program)?;

		let mut not_appliable = HashSet::new();
		program.funcs.iter().filter(|func| func.name.as_str() != "main").for_each(
			|func| {
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
			},
		);

		// let mut killed_global = vec![];

		std::mem::take(&mut program.global_vars).into_iter().for_each(|var| {
			// if var.size() > 4 || not_appliable.contains(&var.ident) || true{
			// 	program.global_vars.push(var);
			// } else {
			// 	killed_global.push(var);
			// }
			program.global_vars.push(var);
		});

		// program
		// 	.funcs
		// 	.iter_mut()
		// 	.filter(|func| func.name.as_str() == "main")
		// 	.for_each(|func| {

		// 	});

		Ok(move_local)
	}
}
