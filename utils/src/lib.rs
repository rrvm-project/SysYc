pub mod errors;
pub mod init_value_item;
pub mod label;
use std::fmt::Display;

pub use errors::*;
pub use init_value_item::*;
pub use label::*;

pub fn fatal_error(str: impl Display) {
	eprintln!("{}: {}", console::style("fatal error").bold().red(), str);
	std::process::exit(0);
}

pub fn warning(str: impl Display) {
	eprintln!("{}: {}", console::style("warning").bold().magenta(), str);
}
