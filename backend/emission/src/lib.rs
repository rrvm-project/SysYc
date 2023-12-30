use ::utils::mapper::LabelMapper;
use rrvm::program::RiscvProgram;

use crate::{label_mapper::map_label, serialize::func_serialize};

mod label_mapper;
mod serialize;
//TODO: 加完整program header,全局变量信息加上
const PROGRAM_HEAD: &str = "  .attribute arch, \"rv64i2p0_m2p0\"\n  .attribute unaligned_access, 0\n  .attribute stack_align, 16\n";

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
