use std::hash::{Hash, Hasher};

use crate::Symbol;

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
