use std::fmt::Display;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Label {
	pub name: String,
}

impl Display for Label {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "%{}", self.name)
	}
}

impl Label {
	pub fn new(name: impl ToString) -> Self {
		Label {
			name: name.to_string(),
		}
	}
}
