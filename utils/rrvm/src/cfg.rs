use std::fmt::Display;

use utils::Label;

pub use crate::basicblock::{BasicBlock, Node};

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
	pub fn make_pretty(&mut self) {
		self.blocks.sort_unstable_by(|x, y| x.borrow().id.cmp(&y.borrow().id));
		self.blocks.iter().for_each(|v| v.borrow_mut().make_pretty())
	}
	pub fn size(&self) -> usize {
		self.blocks.len()
	}
}

pub fn link_node<T>(from: &Node<T>, to: &Node<T>)
where
	T: Display,
{
	if from.borrow().jump_instr.is_none() {
		from.borrow_mut().succ.push(to.clone());
		to.borrow_mut().prev.push(from.clone());
	}
}

pub fn link_cfg<T>(from: &CFG<T>, to: &CFG<T>)
where
	T: Display,
{
	link_node(&from.get_exit(), &to.get_entry())
}
