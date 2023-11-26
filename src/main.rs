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
use namer::visitor::Namer;
use parser::parser::parse;
use typer::visitor::Typer;
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

fn step_llvm(mut program: Program) -> Result<i32> {
	Namer::new().transform(&mut program)?;
	Typer::new().transform(&mut program)?;
	let x = format!("{:#?}", program);
	println!("{}", trans_indent(&x, PARSER_INDENT));
	todo!()
}

#[allow(unused_variables)]
fn step_riscv(program: i32) -> Result<i32> {
	todo!()
	// let mut program = RrvmProgram::new(program);
	// program.solve_global()?;
	// let funcs: Result<Vec<_>, _> =
	// 	program.funcs.into_iter().map(rrvm_func::transform_riscv).collect();
	// program.funcs = funcs?;
	// let code = program.alloc_reg();
	// Ok(code)
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

	let riscv = step_riscv(llvm)?;
	if args.riscv {
		write!(writer, "{:?}", riscv)?;
		return Ok(());
	}

	unreachable!()
}
