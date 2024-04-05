use ::utils::mapper::LabelMapper;
use rrvm::program::RiscvProgram;

use crate::{serialize::func_emission, utils::*};

mod serialize;
mod utils;

pub fn code_emission(program: RiscvProgram, file_name: String) -> String {
	let mut map = LabelMapper::default();
	let funcs = program
		.funcs
		.into_iter()
		.map(func_emission)
		.map(|(name, instrs)| format_func(name, map_label(instrs, &mut map)))
		.collect::<Vec<_>>()
		.join("\n");
	let (bss, data): (Vec<_>, Vec<_>) =
		program.global_vars.into_iter().partition(|v| v.is_bss());
	let data = data.into_iter().map(format_data).collect::<Vec<_>>().join("\n");
	let bss = bss.into_iter().map(format_bss).collect::<Vec<_>>().join("\n");
	format!(
		"{}\n{}{}  .text\n  .global main\n{}\n  .ident {}\n",
		program_head(file_name),
		set_section("  .section	.sbss, \"aw\", @nobits", bss),
		set_section("  .section	.sdata, \"aw\"", data),
		funcs,
		PROGRAM_IDENT
	)
}
