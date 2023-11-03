pub mod errors;

pub use errors::*;

pub fn fatal_error(str: &str) {
	eprintln!("{}: {}", console::style("fatal error").bold().red(), str);
	std::process::exit(0);
}

pub enum Attr {
	Int(i32),
}

pub trait Attrs {
	fn set_attr(&mut self, name: &str, attr: Attr);
	fn get_attr(&self, name: &str) -> Option<&Attr>;
}
