use super::FuyukiLocalValueNumber;

use crate::RrvmOptimizer;
use std::collections::HashMap;

use rrvm::{program::LlvmProgram, LlvmCFG};
use utils::errors::Result;

use llvm::{Temp, Value};

use rrvm::{dominator::naive::compute_dominator, LlvmNode};

use super::traverse;

use super::impl_lvn;

fn solve(cfg: &mut LlvmCFG) -> bool {
	// cfg.analysis();

	let mut subtree: HashMap<i32, Vec<LlvmNode>> = HashMap::new();
	let mut children: HashMap<i32, Vec<LlvmNode>> = HashMap::new();
	let mut father: HashMap<i32, LlvmNode> = HashMap::new();
	compute_dominator(cfg, false, &mut subtree, &mut children, &mut father);
	let (mut post_order_to_block, _id_to_post_order) =
		traverse::init_post(cfg, &subtree, &children, &father);
	let (dfs_order_to_block, _id_to_dfs_order) =
		traverse::init_dfs(cfg, &subtree, &children, &father);

	//TODO: move up

	// lvn: find
	let mut rewirte: HashMap<Temp, Value> = HashMap::new();

	let total = dfs_order_to_block.len();
	for i in 0..total {
		impl_lvn::solve(post_order_to_block.get(&i).unwrap(), &mut rewirte);
	}

	// lvn: rewrite

	let total = dfs_order_to_block.len();
	for i in 0..total {
		impl_lvn::rewrite_block(
			post_order_to_block.get_mut(&i).as_mut().unwrap(),
			&mut rewirte,
		);
	}

	//TODO: move down

	!rewirte.is_empty()
}

impl RrvmOptimizer for FuyukiLocalValueNumber {
	fn new() -> Self {
		FuyukiLocalValueNumber {}
	}

	fn apply(self, program: &mut LlvmProgram) -> Result<bool> {
		Ok(program.funcs.iter_mut().fold(false, |_last, func| solve(&mut func.cfg)))
	}
}
