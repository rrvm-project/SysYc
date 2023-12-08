use std::{collections::HashSet, fmt::Display};

use utils::{InstrTrait, Label, TempTrait};

pub use crate::basicblock::{BasicBlock, Node};
use crate::LlvmCFG;

pub struct CFG<T: InstrTrait<U>, U: TempTrait> {
	pub blocks: Vec<Node<T, U>>,
}

impl<T: InstrTrait<U>, U: TempTrait> CFG<T, U> {
	pub fn new(id: i32, weight: f64) -> Self {
		Self {
			blocks: vec![BasicBlock::new_node(id, weight)],
		}
	}
	pub fn append(&mut self, other: CFG<T, U>) {
		self.blocks.extend(other.blocks);
	}
	pub fn get_entry(&self) -> Node<T, U> {
		self.blocks.first().unwrap().clone()
	}
	pub fn get_exit(&self) -> Node<T, U> {
		self.blocks.last().unwrap().clone()
	}
	pub fn entry_label(&self) -> Label {
		self.get_entry().borrow().label()
	}
	pub fn exit_label(&self) -> Label {
		self.get_exit().borrow().label()
	}
	pub fn make_pretty(&mut self) {
		self.blocks.iter().for_each(|v| v.borrow_mut().make_pretty())
	}
	pub fn size(&self) -> usize {
		self.blocks.len()
	}
	pub fn analysis(&self) {
		self.blocks.iter().for_each(|v| v.borrow_mut().init());
		loop {
			let mut changed = false;
			for u in self.blocks.iter().rev() {
				let mut new_liveout = HashSet::new();
				for v in u.borrow().succ.iter() {
					new_liveout.extend(v.borrow().live_in.clone());
				}
				let uses = u.borrow().uses.clone();
				let defs = u.borrow().defs.clone();
				let mut new_livein: HashSet<_> =
					new_liveout.difference(&defs).cloned().collect();
				new_livein.extend(uses);
				if new_livein != u.borrow().live_in
					|| new_liveout != u.borrow().live_out
				{
					u.borrow_mut().live_in = new_livein;
					u.borrow_mut().live_out = new_liveout;
					changed = true;
				}
			}
			if !changed {
				break;
			}
		}
		self.blocks.iter().for_each(|v| v.borrow_mut().calc_kill());
	}
}

impl LlvmCFG {
	pub fn init_phi(&self) {
		self.blocks.iter().for_each(|v| v.borrow_mut().init_phi());
	}
}

pub fn link_node<T: InstrTrait<U>, U: TempTrait>(
	from: &Node<T, U>,
	to: &Node<T, U>,
) where
	T: Display,
	U: TempTrait,
{
	if from.borrow().jump_instr.is_none() {
		from.borrow_mut().succ.push(to.clone());
		to.borrow_mut().prev.push(from.clone());
	}
}

pub fn link_cfg<T: InstrTrait<U>, U: TempTrait>(
	from: &CFG<T, U>,
	to: &CFG<T, U>,
) where
	T: InstrTrait<U>,
	U: TempTrait,
{
	link_node(&from.get_exit(), &to.get_entry())
}
