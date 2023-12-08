pub mod errors;
pub mod init_value_item;
pub mod label;
pub mod mapper;
pub mod union_find;
use std::{fmt::Display, hash::Hash};

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

pub trait UseTemp<U> {
	fn get_read(&self) -> Vec<U> {
		Vec::new()
	}
	fn get_write(&self) -> Option<U> {
		None
	}
}

pub trait InstrTrait<U>: Display + UseTemp<U> {}
pub trait TempTrait: Display + Hash + Eq + Clone {}

pub fn instr_format<T: Display>(v: T) -> String {
	format!("  {}", v)
}
