use std::fmt::Display;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Label {
	pub name: String,
}

impl Display for Label {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{}", self.name)
	}
}

impl Label {
	pub fn new(name: impl Display) -> Self {
		Label {
			name: name.to_string(),
		}
	}
}

pub fn to_label(id: i32) -> Label {
	match id {
		0 => Label::new("entry"),
		_ => Label::new(format!("B{}", id)),
	}
}
