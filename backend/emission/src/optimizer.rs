use std::collections::HashSet;

use instruction::RiscvInstrSet;

pub fn remove_useless_label(mut instrs: RiscvInstrSet) -> RiscvInstrSet {
	let labels: HashSet<_> =
		instrs.iter().filter_map(|v| v.get_read_label()).collect();
	instrs.retain(|v| v.get_write_label().map_or(true, |v| labels.contains(&v)));
	instrs
}
