use crate::Symbol;

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
	// `ident` is `Some` only when the symbol is global
	pub fn new_symbol<T: Clone>(
		&mut self,
		ident: impl ToString,
		var_type: T,
	) -> Symbol<T> {
		self.cnt += 1;
		Symbol {
			id: self.cnt,
			var_type,
			ident: format!("{} {}", ident.to_string(), self.cnt),
		}
	}
}
