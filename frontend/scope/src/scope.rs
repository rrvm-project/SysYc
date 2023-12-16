use std::collections::HashMap;

use rrvm_symbol::Symbol;
use utils::{errors::Result, SysycError::SyntaxError};

pub struct Scope<T> {
	symbols: HashMap<String, Symbol<T>>,
}

impl<T> Default for Scope<T> {
	fn default() -> Self {
		Self::new()
	}
}

impl<T> Scope<T> {
	pub fn new() -> Self {
		Self {
			symbols: HashMap::new(),
		}
	}
	pub fn get_symbol(&self, ident: &str) -> Option<&Symbol<T>> {
		self.symbols.get(ident)
	}
	pub fn new_symbol(&mut self, ident: &str, symbol: Symbol<T>) -> Result<()> {
		if self.symbols.insert(ident.to_string(), symbol).is_some() {
			Err(SyntaxError(format!("{} is redefinition", ident)))
		} else {
			Ok(())
		}
	}
}
