use llvm::{LlvmTemp, Value};
use rrvm::{
	dominator::compute_dominator, program::LlvmProgram, LlvmCFG, LlvmNode,
};

use super::GLobalValueNumber;

use crate::{
	fuyuki_vn::impl_lvn::{self, SimpleLvnValue},
	RrvmOptimizer,
};

use utils::errors::Result;

use super::stack_hashmap::StackHashMap;

use std::{
	borrow::BorrowMut,
	collections::{HashMap, HashSet},
	vec,
};
use utils::VEC_EXTERN;

fn solve(cfg: &mut LlvmCFG, not_pure: &HashSet<String>) -> bool {
	let mut subtree: HashMap<i32, Vec<LlvmNode>> = HashMap::new();
	let mut children: HashMap<i32, Vec<LlvmNode>> = HashMap::new();
	let mut father: HashMap<i32, LlvmNode> = HashMap::new();
	compute_dominator(cfg, false, &mut subtree, &mut children, &mut father);

	let mut simple_table: StackHashMap<SimpleLvnValue, Value> =
		StackHashMap::new();
	let mut vec_table: StackHashMap<Vec<i32>, Value> = StackHashMap::new();
	let mut temp_to_vec: StackHashMap<LlvmTemp, Vec<i32>> = StackHashMap::new();

	let mut rewirte: HashMap<LlvmTemp, Value> = HashMap::new();

	let mut stack: Vec<Option<LlvmNode>> = vec![Some(cfg.get_entry())];
	while !stack.is_empty() {
		if let Some(item) = stack.pop().unwrap() {
			simple_table.push();
			vec_table.push();
			temp_to_vec.push();
			let block_id = item.as_ref().borrow().id;

			impl_lvn::solve(
				&item,
				&mut rewirte,
				not_pure,
				&mut simple_table,
				&mut vec_table,
				&mut temp_to_vec,
			);

			stack.push(None);
			if let Some(to_append) = children.get(&block_id) {
				for item in to_append {
					stack.push(Some(item.clone()));
				}
			}
		} else {
			simple_table.pop();
			vec_table.pop();
			temp_to_vec.pop();
		}
	}

	for item in &mut cfg.borrow_mut().blocks {
		impl_lvn::rewrite_block(item, &rewirte);
	}

	!rewirte.is_empty()
}

impl RrvmOptimizer for GLobalValueNumber {
	fn new() -> Self {
		GLobalValueNumber {}
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
		Ok(program.funcs.iter_mut().any(|func| solve(&mut func.cfg, &not_pure)))
	}
}
