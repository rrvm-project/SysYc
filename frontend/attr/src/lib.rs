use value::{Value, VarType};

#[derive(Clone, Debug)]
pub enum Attr {
	VarType(VarType),
	Value(Value),
}

pub trait Attrs {
	fn set_attr(&mut self, name: &str, attr: Attr);
	fn get_attr(&self, name: &str) -> Option<&Attr>;
}

impl Attr {
	pub fn to_type(self) -> VarType {
		if let Attr::VarType(v) = self {
			v
		} else {
			unreachable!("Don't downcast if you do not zhe shi shenma leixing")
		}
	}
	pub fn to_value(self) -> Value {
		if let Attr::Value(v) = self {
			v
		} else {
			unreachable!("Don't downcast if you do not zhe shi shenma leixing")
		}
	}
}
