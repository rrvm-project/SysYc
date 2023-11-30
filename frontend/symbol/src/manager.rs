use std::fmt::Display;

use value::{FuncType, VarType};

use crate::{FuncSymbol, Symbol, VarSymbol};

pub struct SymbolManager {
	cnt: i32,
}

impl Default for SymbolManager {
	fn default() -> Self {
		Self::new()
	}
}

impl SymbolManager {
	pub fn new() -> Self {
		Self { cnt: 0 }
	}
	pub fn new_var_symbol(
		&mut self,
		ident: impl Display,
		var_type: VarType,
	) -> VarSymbol {
		self.cnt += 1;
		Symbol {
			id: self.cnt,
			var_type,
			ident: format!("{} {}", ident, self.cnt),
		}
	}
	pub fn new_func_symbol(
		&mut self,
		ident: impl Display,
		var_type: FuncType,
	) -> FuncSymbol {
		self.cnt += 1;
		Symbol {
			id: self.cnt,
			var_type,
			ident: ident.to_string(),
		}
	}
}
