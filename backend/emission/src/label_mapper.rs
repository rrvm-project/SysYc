use instruction::RiscvInstrSet;
use utils::{instr_format, mapper::LabelMapper};

pub fn map_label(mut instrs: RiscvInstrSet, map: &mut LabelMapper) -> String {
	map.map.clear();
	instrs.iter_mut().for_each(|instr| instr.map_label(map));
	instrs.into_iter().map(instr_format).collect::<Vec<_>>().join("\n")
}
