mod cli;

use std::{
	fs::{self, File},
	io,
	io::Write,
};

use anyhow::Result;
use ast::tree::Program;
use clap::Parser;
use cli::Args;
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
fn step_llvm(program: Program) -> Result<()> {
	todo!()
}

#[allow(unused_variables)]
fn step_riscv(what: ()) -> Result<()> {
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
		write!(writer, "{:#?}", program)?;
		return Ok(());
	}

	let llvm = step_llvm(program)?;
	if args.llvm {
		write!(writer, "{:?}", llvm)?;
		return Ok(());
	}

	let riscv = step_riscv(llvm)?;
	if args.riscv {
		write!(writer, "{:?}", riscv)?;
		return Ok(());
	}

	unreachable!()
}
