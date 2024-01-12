use rrvm::LlvmNode;

pub fn solve(block: &mut LlvmNode, father: &mut LlvmNode) {
	block.borrow_mut().update_phi_def();
	let mut defs = block.borrow().phi_defs.clone();

	let mut new_instr = vec![];
	let mut to_father = vec![];
	for instr in &mut block.borrow_mut().instrs {
		let ok_to_jump_up: bool = match instr.get_variant() {
			llvm::LlvmInstrVariant::ArithInstr(_)
			| llvm::LlvmInstrVariant::CompInstr(_)
			| llvm::LlvmInstrVariant::ConvertInstr(_)
			| llvm::LlvmInstrVariant::GEPInstr(_) => {
				let mut ok = true;
				for item in instr.get_read() {
					if defs.contains(&item) {
						ok = false;
						break;
					}
				}
				ok
			}
			// llvm::LlvmInstrVariant::CallInstr(_) => todo!(), // TODO pure func!
			_ => false,
		};
		if !ok_to_jump_up {
			new_instr.push(instr.clone_box()); //怎样不进行复制？
			if let Some(t) = instr.get_write() {
				defs.insert(t);
			}
		} else {
			to_father.push(instr.clone_box());
		}
	}

	block.borrow_mut().instrs = new_instr;

	father.borrow_mut().instrs.append(&mut to_father);
}
