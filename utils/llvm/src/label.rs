use std::fmt::Display;

pub struct Label {
	pub name: String,
}

impl Display for Label {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "%{}", self.name)
	}
}