pub mod errors;
pub use errors::*;
pub mod init_value_item;
pub use init_value_item::*;

pub fn fatal_error(str: &str) {
	eprintln!("{}: {}", console::style("fatal error").bold().red(), str);
	std::process::exit(0);
}
