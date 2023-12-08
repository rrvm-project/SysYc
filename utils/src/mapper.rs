use std::collections::HashMap;

use crate::Label;

#[derive(Default)]
pub struct LabelMapper {
	total: i32,
	pub map: HashMap<Label, Label>,
}

impl LabelMapper {
	pub fn get(&mut self, label: Label) -> Label {
		self
			.map
			.entry(label)
			.or_insert_with(|| {
				self.total += 1;
				Label::new(format!("L_{}", self.total))
			})
			.clone()
	}
}
