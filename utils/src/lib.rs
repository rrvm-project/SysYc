pub mod errors;
pub mod init_value_item;
pub mod label;
pub use errors::*;
pub use init_value_item::*;
pub use label::*;

pub fn fatal_error(str: &str) {
	eprintln!("{}: {}", console::style("fatal error").bold().red(), str);
	std::process::exit(0);
}
