use utils::{InstrTrait, TempTrait};

use crate::cfg::{Node, CFG};

use super::{compute_dominator, compute_dominator_frontier, DomTree};

impl<T: InstrTrait<U>, U: TempTrait> Default for DomTree<T, U> {
	fn default() -> Self {
		Self {
			dominates: Default::default(),
			dominator: Default::default(),
			dom_direct: Default::default(),
			df: Default::default(),
		}
	}
}

impl<T: InstrTrait<U>, U: TempTrait> DomTree<T, U> {
	pub fn new(cfg: &CFG<T, U>, reverse: bool) -> Self {
		let mut dom_tree = Self::default();
		compute_dominator(
			cfg,
			reverse,
			&mut dom_tree.dominates,
			&mut dom_tree.dom_direct,
			&mut dom_tree.dominator,
		);
		compute_dominator_frontier(
			cfg,
			reverse,
			&dom_tree.dominates,
			&dom_tree.dominator,
			&mut dom_tree.df,
		);
		dom_tree
	}
	pub fn get_children(&mut self, id: i32) -> &Vec<Node<T, U>> {
		self.dom_direct.entry(id).or_default()
	}
	pub fn get_df(&mut self, id: i32) -> &Vec<Node<T, U>> {
		self.df.entry(id).or_default()
	}
	pub fn get_dominator(&mut self, id: i32) -> Option<Node<T, U>> {
		self.dominator.get(&id).cloned()
	}
	pub fn dominates(&mut self, id1: i32, id2: i32) -> bool {
		self
			.dominates
			.get(&id1)
			.map_or(false, |v| v.iter().any(|x| x.borrow().id == id2))
	}
}
