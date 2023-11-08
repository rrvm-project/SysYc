use std::collections::HashMap;

use crate::tree::{Break, Continue};

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
