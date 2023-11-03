use std::collections::HashMap;

use crate::tree::{Break, Continue};

impl Continue {
	pub fn new() -> Continue {
		Continue {
			_attrs: HashMap::new(),
		}
	}
}

impl Break {
	pub fn new() -> Break {
		Break {
			_attrs: HashMap::new(),
		}
	}
}
