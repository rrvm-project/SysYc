use std::fmt::Display;

pub use ValueItem::*;

#[derive(Debug)]
pub enum ValueItem {
	Word(u32),
	Zero(usize),
}

#[derive(Debug)]
pub struct GlobalVar {
	pub ident: String,
	pub data: Vec<ValueItem>,
	pub is_float: bool,
	pub is_array: bool,
}

impl Display for ValueItem {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Word(v) => write!(f, "  .word {}", v),
			Zero(v) => write!(f, "  .zero {}", v),
		}
	}
}

impl ValueItem {
	fn size(&self) -> usize {
		match self {
			Word(_) => 4,
			Zero(v) => *v,
		}
	}
}

impl Display for GlobalVar {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let data =
			self.data.iter().map(|v| v.to_string()).collect::<Vec<_>>().join("\n");
		write!(f, "{}:\n{}", self.ident, data)
	}
}

impl GlobalVar {
	pub fn new(
		ident: impl Display,
		data: Vec<ValueItem>,
		is_float: bool,
		is_array: bool,
	) -> Self {
		Self {
			ident: ident.to_string(),
			data,
			is_float,
			is_array,
		}
	}
	pub fn size(&self) -> usize {
		self.data.iter().map(|v| v.size()).sum()
	}
	pub fn is_bss(&self) -> bool {
		matches!(self.data.first(), Some(Zero(_)) if self.data.len() == 1)
	}
}
