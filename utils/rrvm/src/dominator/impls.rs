use crate::{LlvmCFG, LlvmNode};

use super::{compute_dominator, compute_dominator_frontier, DomTree};

impl DomTree {
	pub fn new(cfg: &LlvmCFG, reverse: bool) -> Self {
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
	pub fn get_children(&mut self, id: i32) -> &Vec<LlvmNode> {
		self.dom_direct.entry(id).or_default()
	}
	pub fn get_df(&mut self, id: i32) -> &Vec<LlvmNode> {
		self.df.entry(id).or_default()
	}
}
