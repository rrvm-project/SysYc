use std::collections::HashMap;

use crate::tree::{Break, Continue, LiteralInt, Node};

impl Continue {
	pub fn new() -> Continue {
		Continue {
			_attrs: HashMap::new(),
		}
	}
}

impl Default for Continue {
	fn default() -> Self {
		Self::new()
	}
}

impl Break {
	pub fn new() -> Break {
		Break {
			_attrs: HashMap::new(),
		}
	}
}

impl Default for Break {
	fn default() -> Self {
		Self::new()
	}
}

impl LiteralInt {
	pub fn node(value: i32) -> Node {
		Box::new(Self {
			_attrs: HashMap::new(),
			value,
		})
	}
}
