pub mod constants;
pub mod errors;
pub mod global_var;
pub mod label;
pub mod mapper;
pub mod math;
pub mod union_find;
use std::{fmt::Display, hash::Hash};

pub use constants::*;
pub use errors::*;
pub use global_var::*;
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
pub trait RTN {
	// mem,br,mul/div,floating-point,sum 是这5项的意思
	fn get_rtn_array(&self) -> [i32; 5] {
		[0; 5]
	}
}
pub trait InstrTrait<U>: Display + UseTemp<U> {
	fn is_call(&self) -> bool;
	fn is_branch(&self) -> bool;
}
pub trait TempTrait: Display + Hash + Eq + Clone {}

pub fn instr_format<T: Display>(v: T) -> String {
	format!("  {}", v)
}
