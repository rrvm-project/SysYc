pub mod errors;
pub use errors::*;
use std::collections::HashMap;

pub fn fatal_error(str: &str) {
	eprintln!("{}: {}", console::style("fatal error").bold().red(), str);
	std::process::exit(0);
}

// TODO：这里考虑如何添加symbol
#[derive(Debug, Clone)]
pub enum Attr {
	CompileConstValue(CompileConstValue),
	Symbol(usize),
}

#[derive(Debug, Clone)]
pub enum CompileConstValue {
	Int(i32),
	Float(f32),
	IntArray(HashMap<u32, i32>),
	FloatArray(HashMap<u32, f32>),
}

#[derive(Debug, Clone)]
pub enum InitValueItem {
	Int(i32),
	Float(f32),
	None(usize),
}

pub trait Attrs {
	fn set_attr(&mut self, name: &str, attr: Attr);
	fn get_attr(&self, name: &str) -> Option<&Attr>;
}
