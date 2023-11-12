pub mod errors;
pub use errors::*;
use ir_type::builtin_type::IRType;
use std::collections::HashMap;

pub fn fatal_error(str: &str) {
	eprintln!("{}: {}", console::style("fatal error").bold().red(), str);
	std::process::exit(0);
}
