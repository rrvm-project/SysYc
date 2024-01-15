use std::collections::HashSet;

use rrvm::program::LlvmProgram;

pub fn get_func_list(program: &LlvmProgram) -> HashSet<String> {
	let func_list: HashSet<_> = program
		.funcs
		.iter()
		.filter(|&func| func.can_inline())
		.map(|func| func.name.clone())
		.collect();
	let mut use_list = HashSet::new();
	for func in program.funcs.iter() {
		for block in func.cfg.blocks.iter() {
			for instr in block.borrow().instrs.iter() {
				if instr.is_call() {
					use_list.insert(instr.get_label().name);
				}
			}
		}
	}
	func_list.intersection(&use_list).cloned().collect()
}
