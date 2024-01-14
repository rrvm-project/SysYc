use std::collections::HashMap;

use llvm::{ArithInstr, LlvmTemp, Value};
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
		let mut map = HashMap::new();
		for (target, value) in src.iter() {
			map.insert(target, (value, 0));
		}
		for (_, value) in src.iter() {
			if let Value::Temp(temp) = value {
				map.entry(temp).and_modify(|(_, v)| *v += 1);
			}
		}
		let mut ready = Vec::<(&LlvmTemp, &Value)>::new();
		map.retain(|target, (value, cnt)| {
			*cnt != 0 || {
				ready.push((target, value));
				false
			}
		});
		while let Some((target, value)) = ready.pop() {
			let var_type = value.get_type();
			v.borrow_mut().push(ArithInstr::new(
				target.clone(),
				var_type.default_value(),
				var_type.move_op(),
				value.clone(),
				var_type,
			));
			if let Value::Temp(target) = value {
				map.entry(target).and_modify(|(value, cnt)| {
					*cnt -= 1;
					if *cnt == 0 {
						ready.push((target, value));
					}
				});
			}
		}
	}
	u.borrow_mut().phi_instrs.clear();
}
