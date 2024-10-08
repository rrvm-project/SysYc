pub use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
	#[arg(long)]
	pub parse: bool,

	#[arg(short = 'S')]
	pub assemble: bool,

	#[arg(long)]
	pub llvm: bool,

	#[arg(long)]
	pub riscv: bool,

	#[arg(short)]
	pub output: Option<String>,

	#[arg(short = 'O')]
	pub opimizer: Option<i32>,

	#[arg(value_parser)]
	pub input: Option<String>,
}
