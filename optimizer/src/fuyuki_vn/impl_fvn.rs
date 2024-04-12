use super::FuyukiLocalValueNumber;

use crate::{
	fuyuki_vn::{impl_down, stack_hashmap::StackHashMap},
	RrvmOptimizer,
};
use std::collections::{HashMap, HashSet};
use utils::{UseTemp, VEC_EXTERN};

use rrvm::{program::LlvmProgram, LlvmCFG};
use utils::errors::Result;

use llvm::{LlvmTemp, Value};

use rrvm::{dominator::naive::compute_dominator, LlvmNode};

use super::traverse;

use super::{impl_lvn, impl_up};

use super::fvn_utils::MaxMin;

use crate::fuyuki_vn::impl_lvn::SimpleLvnValue;

fn solve(cfg: &LlvmCFG, not_pure: &HashSet<String>) -> bool {
	// cfg.analysis();

	let mut subtree: HashMap<i32, Vec<LlvmNode>> = HashMap::new();
	let mut children: HashMap<i32, Vec<LlvmNode>> = HashMap::new();
	let mut father: HashMap<i32, LlvmNode> = HashMap::new();
	compute_dominator(cfg, false, &mut subtree, &mut children, &mut father);
	let (mut post_order_to_block, _id_to_post_order) =
		traverse::init_post(cfg, &subtree, &children, &father);

	//move up

	let total = post_order_to_block.len();
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
	let mut rewirte: HashMap<LlvmTemp, Value> = HashMap::new();
	let mut simple_table: StackHashMap<SimpleLvnValue, Value> =
		StackHashMap::new();
	let mut vec_table: StackHashMap<Vec<i32>, Value> = StackHashMap::new();
	let mut temp_to_vec: StackHashMap<LlvmTemp, Vec<i32>> = StackHashMap::new();

	let total = post_order_to_block.len();
	for i in 0..total {
		// Actaully, this is LVN
		simple_table.push();
		vec_table.push();
		temp_to_vec.push();

		impl_lvn::solve(
			post_order_to_block.get(&i).unwrap(),
			&mut rewirte,
			not_pure,
			&mut simple_table,
			&mut vec_table,
			&mut temp_to_vec,
		);

		simple_table.pop();
		vec_table.pop();
		temp_to_vec.pop();
	}
	// lvn: rewrite

	let total = post_order_to_block.len();
	for i in 0..total {
		impl_lvn::rewrite_block(
			post_order_to_block.get_mut(&i).as_mut().unwrap(),
			&rewirte,
		);
	}
	//move down

	let mut weights = HashMap::new();
	let mut uses: HashMap<LlvmTemp, MaxMin<usize>> = HashMap::new();

	fn update_use(
		uses: &mut HashMap<LlvmTemp, MaxMin<usize>>,
		i: usize,
		temp: LlvmTemp,
	) {
		if let Some(use_item) = uses.get_mut(&temp) {
			use_item.update(i);
		} else {
			uses.insert(temp.clone(), MaxMin::new_with_init(i));
		}
	}

	let (mut dfs_order_to_block, id_to_dfs_order) =
		traverse::init_dfs(cfg, &subtree, &children, &father);

	let total = dfs_order_to_block.len();

	let mut dfs_order_to_id: HashMap<usize, i32> = HashMap::new();
	for (key, value) in id_to_dfs_order.iter() {
		dfs_order_to_id.insert(*value, *key);
	}

	for i in 0..total {
		let current = dfs_order_to_block.get(&i).unwrap();
		weights.insert(current.borrow().id, current.borrow().weight);

		// 这里i是dfs序！
		for instr in &current.borrow().phi_instrs {
			let father_id = father
				.get(dfs_order_to_id.get(&i).unwrap())
				.unwrap()
				.as_ref()
				.borrow()
				.id;
			let father_dfs = *id_to_dfs_order.get(&father_id).unwrap();
			for temp in instr.get_read() {
				update_use(&mut uses, father_dfs, temp);
			}
		}

		for instr in &current.borrow().instrs {
			for temp in instr.get_read() {
				update_use(&mut uses, i, temp);
			}
		}

		for instr in &current.borrow().jump_instr {
			for temp in instr.get_read() {
				update_use(&mut uses, i, temp);
			}
		}
	}

	let mut known_lca: HashMap<(usize, usize), usize> = HashMap::new();

	impl_down::solve(
		&mut uses,
		&mut known_lca,
		&mut dfs_order_to_block,
		&id_to_dfs_order,
		&dfs_order_to_id,
		&father,
	);
	!rewirte.is_empty()
}

impl RrvmOptimizer for FuyukiLocalValueNumber {
	fn new() -> Self {
		FuyukiLocalValueNumber {}
	}

	fn apply(self, program: &mut LlvmProgram) -> Result<bool> {
		let mut not_pure = HashSet::new();

		for item in VEC_EXTERN {
			not_pure.insert(item.to_string());
		}

		program
			.funcs
			.iter()
			.map(|func| {
				if !func.external_resorce.is_empty() {
					not_pure.insert(func.name.clone());
				}
			})
			.count();
		Ok(program.funcs.iter_mut().any(|func| solve(&func.cfg, &not_pure)))
	}
}
