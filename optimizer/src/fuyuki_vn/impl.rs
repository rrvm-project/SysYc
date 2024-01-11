use super::FuyukiLocalValueNumber;

use crate::RrvmOptimizer;
use std::collections::HashMap;

use rrvm::{program::LlvmProgram, LlvmCFG};
use utils::errors::Result;

use llvm::{Temp, Value};

use rrvm::{dominator::naive::compute_dominator, LlvmNode};

use super::traverse;

use super::{impl_lvn, impl_up};

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

	//move up

	let total = dfs_order_to_block.len();
	for i in 0..total {
		let current = post_order_to_block.get_mut(&i).unwrap();
		let block_id = current.borrow().id;
		if let Some(father_node) = father.get_mut(&block_id) {
			// dbg!(block_id, father_node.borrow().id);
			father_node.borrow_mut().init_data_flow();
			father_node.borrow_mut().update_phi_def();
			// dbg!(&father_node.borrow().defs, &father_node.borrow().phi_defs);
			impl_up::solve(current, father_node);
		// println!("{:?} {:?}", block_id, father_node.borrow().id);
		} else {
			break;
		};

		//
	}

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
	// let total = dfs_order_to_block.len();
	// for i in 0..total {
	// 	let current = dfs_order_to_block.get_mut(&i).unwrap();
	// 	// dbg!(current.borrow().id, current.borrow().weight);

	// 	//
	// }

	!rewirte.is_empty()
}

impl RrvmOptimizer for FuyukiLocalValueNumber {
	fn new() -> Self {
		FuyukiLocalValueNumber {}
	}

	fn apply(self, program: &mut LlvmProgram) -> Result<bool> {
		Ok(program.funcs.iter_mut().any(|func| solve(&mut func.cfg)))
	}
}
