pub mod impls;
pub mod tree;
pub mod visitor;

pub use tree::*;
use value::Value;
pub use visitor::*;

pub use tree::AstRetType::*;

pub fn val_2_node(value: &Value) -> Node {
	match value {
		Value::Int(v) => LiteralInt::node(*v),
		Value::Float(v) => LiteralFloat::node(*v),
		_ => unreachable!(),
	}
}

pub fn shirink(node: &mut Node) {
	if let Some(value) = node.get_attr("value") {
		let v = value.clone();
		*node = val_2_node(&value.into());
		node.set_attr("value", v);
	}
}
