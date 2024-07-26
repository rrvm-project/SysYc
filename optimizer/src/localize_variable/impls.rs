use super::LocalizeVariable;
use std::collections::HashSet;

use crate::RrvmOptimizer;
use llvm::{
	AllocInstr, LlvmInstrTrait, LlvmInstrVariant, LlvmTemp, Value, VarType,
};
use rrvm::program::LlvmProgram;
use utils::{errors::Result, GlobalVar};

impl RrvmOptimizer for LocalizeVariable {
	fn new() -> Self {
		LocalizeVariable {}
	}

	fn apply(self, program: &mut LlvmProgram) -> Result<bool> {
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

		let mut killed_global = vec![];

		std::mem::take(&mut program.global_vars).into_iter().for_each(|var| {
			if var.size() > 4 || not_appliable.contains(&var.ident) {
				program.global_vars.push(var);
			} else {
				killed_global.push(var);
			}
		});

		fn initalize_variable(
			g: GlobalVar,
			program: &mut LlvmProgram,
		) -> Vec<Box<dyn LlvmInstrTrait>> {
			let mut result: Vec<Box<dyn LlvmInstrTrait>> = vec![];
			let var_type = if g.is_float {
				VarType::F32Ptr
			} else {
				VarType::I32Ptr
			};
			let length = g.size();
			assert!(length == 4);// Do not support array here!


			// result.push(Box::new(AllocInstr {
			// 	target: LlvmTemp {
			// 		name: g.ident,
			// 		is_global: false,
			// 		var_type,
			// 	},
			// 	var_type,
			// 	length,
			// }));mainmai

			result
		}

		program
			.funcs
			.iter_mut()
			.filter(|func| func.name.as_str() == "main")
			.for_each(|func| {
				// func.cfg.get_entry().borrow_mut().instrs.append(
				//     // killed_global

				// );

				// func.cfg.blocks.iter_mut().for_each(|block|{
				//     block.borrow().instrs.iter_mut().for_each(|instr|{
				//         if let LlvmInstrVariant::LoadInstr(instr) = instr.get_variant(){
				//             if let Value::Temp(t) = &instr.addr{
				//                 if t.is_global {
				//                     not_appliable.insert(t.clone());
				//                 }
				//             }
				//         }
				//     })
				// })
			});

		Ok(false)
	}
}
