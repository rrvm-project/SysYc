use std::collections::HashMap;

use rrvm::cfg::CFG;

use crate::{
	tree::{Break, Continue, LiteralFloat, LiteralInt, Node},
	AstRetType,
};

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

impl LiteralFloat {
	pub fn node(value: f32) -> Node {
		Box::new(Self {
			_attrs: HashMap::new(),
			value,
		})
	}
}

impl AstRetType {
	pub fn unwarp_cfg(self) -> CFG {
		match self {
			Self::Cfg(v) => v,
			_ => unreachable!(),
		}
	}
}
