use std::fmt::Display;

use utils::Label;

use crate::basicblock::{BasicBlock, Node};

pub struct CFG<T: Display> {
	pub blocks: Vec<Node<T>>,
}

impl<T: Display> CFG<T> {
	pub fn new(id: i32) -> Self {
		Self {
			blocks: vec![BasicBlock::new_node(id)],
		}
	}
	pub fn append(&mut self, other: CFG<T>) {
		self.blocks.extend(other.blocks);
	}
	pub fn get_entry(&self) -> Node<T> {
		self.blocks.first().unwrap().clone()
	}
	pub fn get_exit(&self) -> Node<T> {
		self.blocks.last().unwrap().clone()
	}
	pub fn entry_label(&self) -> Label {
		self.get_entry().borrow().label()
	}
	pub fn exit_label(&self) -> Label {
		self.get_exit().borrow().label()
	}
}

pub fn link_basic_block<T>(from: Node<T>, to: Node<T>)
where
	T: Display,
{
	from.borrow_mut().succ.push(to.clone());
	to.borrow_mut().prev.push(from.clone());
}

pub fn link_cfg<T>(from: &mut CFG<T>, to: &mut CFG<T>)
where
	T: Display,
{
	link_basic_block(from.get_exit(), to.get_entry())
}
