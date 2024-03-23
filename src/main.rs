mod cli;
mod config;
mod logging;
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
#[cfg(not(feature = "simu"))]
use emission::code_emission;
#[cfg(not(feature = "simu"))]
use instruction::temp::TempManager;
use irgen::IRGenerator;
use log::trace;
use namer::visitor::Namer;
use optimizer::*;
use parser::parser::parse;
#[cfg(not(feature = "simu"))]
use register::solve_register;
use rrvm::program::*;
#[cfg(not(feature = "simu"))]
use transform::get_functions;
use typer::visitor::Typer;
use utils::{fatal_error, map_sys_err, warning};

fn step_parse(file_name: &str) -> Result<Program> {
	let code = fs::read_to_string(file_name)
		.map_err(|_| fatal_error("no input files"))
		.unwrap();
	Ok(parse(&code)?)
}

fn step_llvm(mut program: Program, level: i32) -> Result<LlvmProgram> {
	Namer::default().transform(&mut program)?;
	Typer::default().transform(&mut program)?;
	let mut program = IRGenerator::new().to_rrvm(program)?;
	match level {
		0 => Optimizer0::new().apply(&mut program)?,
		1 => Optimizer1::new().apply(&mut program)?,
		2 => Optimizer2::new().apply(&mut program)?,
		_ => {
			warning(format!(
				"optimization level '-O{level}' is not supported; using '-O0' instead",
			));
			Optimizer0::new().apply(&mut program)?
		}
	};
	Ok(program)
}

#[cfg(not(feature = "simu"))]
fn step_riscv(program: LlvmProgram, level: i32) -> Result<RiscvProgram> {
	use backend_optimizer::backend_optimize;

	let mut riscv_program = RiscvProgram::new(TempManager::default());
	riscv_program.global_vars = program.global_vars;
	get_functions(&mut riscv_program, program.funcs)?;
	solve_register(&mut riscv_program);
	backend_optimize(&mut riscv_program, level);
	Ok(riscv_program)
}

fn main() -> Result<()> {
	logging::init();
	trace!("start");
	let args = Args::parse();

	let mut writer: Box<dyn Write> = if let Some(o) = args.output {
		Box::new(File::create(o).map_err(map_sys_err)?)
	} else {
		Box::new(io::stdout())
	};

	let level = args.opimizer.unwrap_or(0);

	let file_name = args.input.unwrap_or_else(|| {
		fatal_error("no input files");
		unreachable!()
	});

	let program = step_parse(&file_name)?;
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

	#[cfg(feature = "simu")]
	{
		use simulator::simulator;
		use std::io::Read;
		println!("working as a simulator!");

		let mut input_content;

		if let Some(input_filename) = args.simu_input {
			let mut file = File::open(input_filename)?;
			input_content = String::new();
			file.read_to_string(&mut input_content)?;
		} else {
			input_content = "".to_string();
		}
		dbg!(&input_content);
		let mut simu = simulator::MiddleSimulator::new(input_content);
		simu.run_program(&llvm);
		dbg!(&simu.output);
		dbg!(&simu.return_scratch);
	}

	#[cfg(not(feature = "simu"))]
	{
		let riscv = step_riscv(llvm, level)?;
		if args.riscv {
			write!(writer, "{}", riscv)?;
			return Ok(());
		}

		let code = code_emission(riscv, file_name);
		write!(writer, "{}", code)?;
	}

	Ok(())
}
