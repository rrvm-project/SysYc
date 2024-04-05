#[derive(PartialEq, Eq, Hash, Debug)]

pub enum ExternalResource {
	Memory,
	Call(String),
	CallExtern,
}
