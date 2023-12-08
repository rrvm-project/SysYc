use std::collections::HashMap;

use crate::tree::{LiteralFloat, LiteralInt, Node};

impl LiteralInt {
	pub fn node(value: i32) -> Node {
		Box::new(Self {
			_attrs: HashMap::new(),
			value,
		})
	}
}

impl LiteralFloat {
	pub fn node(value: f32) -> Node {
		Box::new(Self {
			_attrs: HashMap::new(),
			value,
		})
	}
}
