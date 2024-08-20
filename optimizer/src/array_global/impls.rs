use super::ArrayGlobal;

use crate::{metadata::MetaData, RrvmOptimizer};
use llvm::{LlvmInstrVariant, LlvmTemp, LoadInstr, Value};
use rrvm::program::LlvmProgram;
use utils::{errors::Result, GlobalVar, ValueItem::Zero};

fn move_local_array(program: &mut LlvmProgram) -> Result<bool> {
	let changed = false;
	let mut new_global = vec![];
	program.funcs.iter_mut().filter(|func| func.name == "main").for_each(
		|func| {
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
			}
		},
	);

	program.global_vars.append(&mut new_global);

	Ok(changed)
}

impl RrvmOptimizer for ArrayGlobal {
	fn new() -> Self {
		ArrayGlobal {}
	}

	fn apply(
		self,
		program: &mut LlvmProgram,
		_meta: &mut MetaData,
	) -> Result<bool> {
		program.analysis();
		let move_local_array = move_local_array(program)?;
		program.analysis();
		Ok(move_local_array)
	}
}
