#[derive(Clone, Debug)]
pub enum LlvmAttr {
	Mark,
}

pub trait LlvmAttrs {
	fn set_attr(&mut self, name: &str, attr: LlvmAttr);
	fn get_attr(&self, name: &str) -> Option<&LlvmAttr>;
	fn clear_attr(&mut self, name: &str);
}
