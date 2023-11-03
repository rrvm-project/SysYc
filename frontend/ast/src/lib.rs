pub mod tree;
pub mod visitor;

mod impls;

#[derive(Debug)]
pub enum Type {
	Int,
	Float,
}
