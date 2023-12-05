use std::collections::HashMap;

use llvm::ArithInstr;
use rrvm::LlvmNode;

pub fn remove_phi(u: &LlvmNode) {
	if u.borrow().no_phi() {
		return;
	}
	let mut table = HashMap::new();
	for instr in u.borrow().phi_instrs.iter() {
		for (value, label) in instr.source.iter() {
			table
				.entry(label.clone())
				.or_insert_with(Vec::new)
				.push((instr.target.clone(), value.clone()));
		}
	}
	let prev = u.borrow().prev.clone();
	for v in prev.into_iter() {
		let src = table.remove(&v.borrow().label()).unwrap();
		for (target, value) in src {
			let var_type = value.get_type();
			v.borrow_mut().push(Box::new(ArithInstr {
				target,
				lhs: var_type.default_value(),
				op: var_type.move_op(),
				var_type,
				rhs: value,
			}));
		}
	}
	u.borrow_mut().phi_instrs.clear();
}
