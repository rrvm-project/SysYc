use super::fvn_utils::MaxMin;
use llvm::{LlvmInstrTrait, LlvmTemp};
use rrvm::LlvmNode;
use std::collections::HashMap;

pub fn get_father_id(a: i32, father: &HashMap<i32, LlvmNode>) -> Option<i32> {
	if let Some(father) = father.get(&a) {
		father.as_ref().borrow().id.into()
	} else {
		None
	}
}

pub fn get_lca(
	mut a: i32,
	mut b: i32,
	father: &HashMap<i32, LlvmNode>,
) -> Option<i32> {
	let head_a = a;
	let head_b = b;
	loop {
		if a == b {
			break;
		};
		let mut flag = false;
		a = if let Some(father) = get_father_id(a, father) {
			father
		} else {
			flag = true;
			head_b
		};
		b = if let Some(father) = get_father_id(b, father) {
			father
		} else {
			if flag {
				return None;
			}
			head_a
		};
	}
	Some(a)
}

pub fn solve(
	uses: &mut HashMap<LlvmTemp, MaxMin<usize>>,
	known_lca: &mut HashMap<(usize, usize), usize>,
	dfs_order_to_block: &mut HashMap<usize, LlvmNode>,
	id_to_dfs_order: &HashMap<i32, usize>,
	dfs_order_to_id: &HashMap<usize, i32>,
	father: &HashMap<i32, LlvmNode>,
) {
	let mut lazy_insert: HashMap<usize, Vec<Box<dyn LlvmInstrTrait>>> =
		HashMap::new();

	let mut dfs_order_to_weight: HashMap<usize, f64> = HashMap::new();

	let mut i = 0;

	while let Some(block) = dfs_order_to_block.get_mut(&i) {
		dfs_order_to_weight.insert(i, block.as_ref().borrow().weight);
		i += 1;
	}

	i = 0;

	while let Some(block) = dfs_order_to_block.get_mut(&i) {
		let mut new_instrs = vec![];
		let weight = block.as_ref().borrow().weight;

		for instr in &block.as_ref().borrow().instrs {
			let target = match instr.get_variant() {
				llvm::LlvmInstrVariant::ArithInstr(instr) => Some(instr.target.clone()),
				llvm::LlvmInstrVariant::CompInstr(instr) => Some(instr.target.clone()),
				llvm::LlvmInstrVariant::ConvertInstr(instr) => {
					Some(instr.target.clone())
				}
				llvm::LlvmInstrVariant::GEPInstr(instr) => Some(instr.target.clone()),
				_ => None,
			};

			let move_to = if let Some(target) = &target {
				if let Some(uses) = uses.get(target) {
					let max = uses.max();
					let min = uses.min();
					let lca = if let Some(lca) = known_lca.get(&(max, min)) {
						*lca
					} else {
						let max_blockid = *dfs_order_to_id.get(&max).unwrap();
						let min_blockid = *dfs_order_to_id.get(&min).unwrap();

						let lca_blockid =
							get_lca(max_blockid, min_blockid, father).unwrap();
						let lca = id_to_dfs_order.get(&lca_blockid).unwrap();
						known_lca.insert((max, min), *lca);
						*lca
					};
					//println!("");
					let (mut min_wieght, mut min_dfs_num) = (weight, i);
					//println!("{:?}", (min_dfs_num, min_wieght));
					let mut ptr_id = *dfs_order_to_id.get(&lca).unwrap();
					let mut ptr_dfs_id;
					loop {
						ptr_dfs_id = *id_to_dfs_order.get(&ptr_id).unwrap();
						if ptr_dfs_id <= i {
							break;
						}
						let weight = *dfs_order_to_weight.get(&ptr_dfs_id).unwrap();

						//println!("{:?}", (ptr_dfs_id, weight));

						if weight < min_wieght {
							min_wieght = weight;
							min_dfs_num = ptr_dfs_id;
						}

						if let Some(next) = get_father_id(ptr_id, father) {
							ptr_id = next;
						} else {
							break;
						}
					}
					if ptr_dfs_id == i && min_dfs_num != i {
						Some(min_dfs_num)
					} else {
						None
					}
				} else {
					None
				}
			} else {
				None
			};

			if let Some(move_to) = move_to {
				//println!("move to {:?}", move_to);
				if let Some(to_insert) = lazy_insert.get_mut(&move_to) {
					to_insert.push(instr.clone());
				} else {
					lazy_insert.insert(move_to, vec![instr.clone()]);
				}
				uses.get_mut(&target.unwrap()).unwrap().update(move_to);
			} else {
				new_instrs.push(instr.clone());
			}
		}

		if let Some(lazy_insert) = lazy_insert.get_mut(&i) {
			block.borrow_mut().instrs = std::mem::take(lazy_insert);
			block.borrow_mut().instrs.append(&mut new_instrs);
		} else {
			block.borrow_mut().instrs = std::mem::take(&mut new_instrs);
		}

		i += 1
	}
}
