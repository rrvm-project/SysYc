use rrvm::program::RiscvProgram;

pub fn remove_useless_instr(program: &mut RiscvProgram) {
	for func in program.funcs.iter_mut() {
		for block in func.cfg.blocks.iter() {
			block.borrow_mut().instrs.retain(|v| !v.useless());
		}
	}
}
