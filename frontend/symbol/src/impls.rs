use std::hash::{Hash, Hasher};

use crate::{FuncSymbol, Symbol};

use utils::constants::{VEC_EXTERN, VEC_MACRO};

impl<T> PartialEq for Symbol<T> {
	fn eq(&self, other: &Self) -> bool {
		self.id == other.id
	}
}

impl<T> Eq for Symbol<T> {}

impl<T> Hash for Symbol<T> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.id.hash(state);
	}
}

impl FuncSymbol {
	pub fn is_extern(&self) -> bool {
		VEC_EXTERN.contains(&self.ident.as_str())
	}
	pub fn is_macro(&self) -> bool {
		VEC_MACRO.contains(&self.ident.as_str())
	}
}
