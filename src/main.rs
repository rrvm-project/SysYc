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
use emission::code_emission;
use irgen::IRGenerator;
use namer::visitor::Namer;
use optimizer::*;
use parser::parser::parse;
use register::register_alloc;
use rrvm::program::*;
use transform::convert_func;
use typer::visitor::Typer;
use utils::{fatal_error, map_sys_err, warning};

fn step_parse(name: Option<String>) -> Result<Program> {
	if name.is_none() {
		fatal_error("no input files");
	}
	let code = fs::read_to_string(name.unwrap())
		.map_err(|_| fatal_error("no input files"))
		.unwrap();
	Ok(parse(&code)?)
}

fn step_llvm(mut program: Program, level: i32) -> Result<LlvmProgram> {
	Namer::default().transform(&mut program)?;
	Typer::default().transform(&mut program)?;
	let mut program = IRGenerator::default().to_rrvm(program)?;
	match level {
		0 => Optimizer0::new().apply(&mut program)?,
		1 => Optimizer1::new().apply(&mut program)?,
		_ => {
			warning(format!(
				"optimization level '-O{level}' is not supported; using '-O0' instead",
			));
			Optimizer0::new().apply(&mut program)?
		}
	};
	Ok(program)
}

fn step_riscv(program: LlvmProgram, _level: i32) -> Result<RiscvProgram> {
	let mut riscv_program = RiscvProgram::new();
	for func in program.funcs.into_iter() {
		riscv_program.funcs.push(convert_func(func)?);
	}
	riscv_program.funcs.iter_mut().for_each(register_alloc);
	Ok(riscv_program)
}

fn main() -> Result<()> {
	let args = Args::parse();

	let mut writer: Box<dyn Write> = if let Some(o) = args.output {
		Box::new(File::create(o).map_err(map_sys_err)?)
	} else {
		Box::new(io::stdout())
	};

	let level = args.opimizer.unwrap_or(0);

	let program = step_parse(args.input)?;
	if args.parse {
		let x = format!("{:#?}", program);
		write!(writer, "{}", trans_indent(&x, PARSER_INDENT))?;
		return Ok(());
	}

	let llvm = step_llvm(program, level)?;
	if args.llvm {
		write!(writer, "{}", llvm)?;
		return Ok(());
	}

	let riscv = step_riscv(llvm, level)?;
	if args.riscv {
		write!(writer, "{}", riscv)?;
		return Ok(());
	}

	let code = code_emission(riscv);
	write!(writer, "{}", code)?;

	Ok(())
}
