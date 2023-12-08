use ::utils::mapper::LabelMapper;
use rrvm::program::RiscvProgram;

use crate::{label_mapper::map_label, serialize::func_serialize};

mod label_mapper;
mod serialize;

const PROGRAM_HEAD: &str = "  .text\n  .align 1\n  .globl main\n";

pub fn code_emission(program: RiscvProgram) -> String {
	let mut map = LabelMapper::default();
	let funcs = program
		.funcs
		.into_iter()
		.map(func_serialize)
		.map(|(name, instrs)| format!("{name}:\n{}", map_label(instrs, &mut map)))
		.collect::<Vec<_>>()
		.join("\n");
	format!("{}{}\n", PROGRAM_HEAD, funcs)
}
