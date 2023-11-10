pub mod errors;
pub use errors::*;

pub fn fatal_error(str: &str) {
	eprintln!("{}: {}", console::style("fatal error").bold().red(), str);
	std::process::exit(0);
}
