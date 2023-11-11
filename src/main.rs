mod cli;
mod config;
mod printer;

use std::{
	fs::{self, File},
	io,
	io::Write,
};

use crate::{config::PARSER_INDENT, printer::trans_indent};
use anyhow::Result;
use ast::tree::Program;
use clap::Parser;
use cli::Args;
use ir_gen::llvmirgen::LlvmIrGen;
use llvm::LlvmProgram;
use namer::namer::Namer;
use parser::parser::parse;
use utils::{fatal_error, map_sys_err};

fn step_parse(name: Option<String>) -> Result<Program> {
	if name.is_none() {
		fatal_error("no input files");
	}
	let code = fs::read_to_string(name.unwrap())
		.map_err(|_| fatal_error("no input files"))
		.unwrap();
	Ok(parse(&code)?)
}

#[allow(unused_variables)]
fn step_llvm(program: Program) -> Result<LlvmProgram> {
	let mut namer: Namer = Namer::default();
	let (program, data) = namer.transform(program)?;

	// let mut writer: Box<dyn Write> = Box::new(io::stdout());
	// let x = format!("{:#?}", program);
	// write!(writer, "{}", trans_indent(&x, PARSER_INDENT))?;

	println!("Data From Namer \n {:?}", data);

	let mut generator: LlvmIrGen = LlvmIrGen {
		data,
		funcs: vec![],
		funcemitter: None,
	};
	generator.transform(program)?;
	Ok(generator.emit_program())
}

#[allow(unused_variables)]
fn step_riscv(what: i32) -> Result<i32> {
	todo!()
}

fn main() -> Result<()> {
	let args = Args::parse();

	let mut writer: Box<dyn Write> = if let Some(o) = args.output {
		Box::new(File::create(o).map_err(map_sys_err)?)
	} else {
		Box::new(io::stdout())
	};

	let program = step_parse(args.input)?;
	if args.parse {
		let x = format!("{:#?}", program);
		write!(writer, "{}", trans_indent(&x, PARSER_INDENT))?;
		return Ok(());
	}

	let llvm = step_llvm(program)?;
	if args.llvm {
		write!(writer, "{}", llvm)?;
		return Ok(());
	}

	let riscv = step_riscv(1)?;
	if args.riscv {
		write!(writer, "{:?}", riscv)?;
		return Ok(());
	}

	unreachable!()
}
