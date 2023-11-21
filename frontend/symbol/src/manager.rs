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
	/// `ident` is `Some` only when the symbol is global
	/// cyh: this rule seems to be violated in namer
	pub fn new_symbol<T: Clone>(
		&mut self,
		ident: Option<String>,
		var_type: T,
	) -> Symbol<T> {
		self.cnt += 1;
		Symbol {
			id: self.cnt,
			var_type,
			is_global: false,
			ident: ident.unwrap_or(self.cnt.to_string()),
		}
	}
}
