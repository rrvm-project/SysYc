use std::fmt::Display;

#[derive(Clone,PartialEq, Eq, PartialOrd, Ord)]
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

#[derive(Default)]
pub struct LabelManager {
	total: u32,
}

impl LabelManager {
	pub fn new() -> Self {
		Self::default()
	}
	pub fn new_label(&mut self) -> Label {
		self.total += 1;
		Label::new("L".to_string() + self.total.to_string().as_str())
	}
}
