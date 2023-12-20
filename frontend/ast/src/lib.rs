pub mod impls;
pub mod tree;
pub mod visitor;

pub use tree::*;
use value::Value;
pub use visitor::*;

pub fn val_2_node(value: &Value) -> Node {
	match value {
		Value::Int(v) => LiteralInt::node(*v),
		Value::Float(v) => LiteralFloat::node(*v),
		_ => unreachable!(),
	}
}

pub fn shirink(node: &mut Node) {
	if let Some(value) = node.get_attr("value") {
		match value {
			attr::Attr::Value(Value::Float(_)) | attr::Attr::Value(Value::Int(_)) => {
				let v = value.clone();
				*node = val_2_node(&value.into());
				node.set_attr("value", v);
			}
			_ => {}
		}
	}
}
