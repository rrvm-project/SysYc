use std::collections::HashMap;

use rrvm::LlvmCFG;

use rrvm::LlvmNode;

pub fn init_post(
	cfg: &LlvmCFG,
	_subtree: &HashMap<i32, Vec<LlvmNode>>,
	children: &HashMap<i32, Vec<LlvmNode>>,
	_father: &HashMap<i32, LlvmNode>,
) -> (HashMap<usize, LlvmNode>, HashMap<i32, usize>) {
	let mut post_order_to_block: HashMap<usize, LlvmNode> = HashMap::new();
	let mut id_to_post_order: HashMap<i32, usize> = HashMap::new();

	let mut stack: Vec<(LlvmNode, bool)> = vec![(cfg.get_entry(), false)];

	while !stack.is_empty() {
		let id = post_order_to_block.len();
		let (item, finish) = stack.pop().unwrap();

		let block_id = item.as_ref().borrow().id;

		if finish {
			post_order_to_block.insert(id, item.clone());
			id_to_post_order.insert(item.as_ref().borrow().id, id);
			continue;
		}
		stack.push((item.clone(), true));

		if let Some(to_append) = children.get(&block_id) {
			for item in to_append {
				stack.push((item.clone(), false));
			}
		}
	}

	(post_order_to_block, id_to_post_order)
}

pub fn init_dfs(
	cfg: &LlvmCFG,
	_subtree: &HashMap<i32, Vec<LlvmNode>>,
	children: &HashMap<i32, Vec<LlvmNode>>,
	_father: &HashMap<i32, LlvmNode>,
) -> (HashMap<usize, LlvmNode>, HashMap<i32, usize>) {
	let mut dfs_order_to_block: HashMap<usize, LlvmNode> = HashMap::new();
	let mut id_to_dfs_order: HashMap<i32, usize> = HashMap::new();

	let mut stack: Vec<LlvmNode> = vec![cfg.get_entry()];

	while !stack.is_empty() {
		let id = dfs_order_to_block.len();
		let item = stack.pop().unwrap();

		let block_id = item.as_ref().borrow().id;

		dfs_order_to_block.insert(id, item.clone());
		id_to_dfs_order.insert(item.as_ref().borrow().id, id);

		if let Some(to_append) = children.get(&block_id) {
			for item in to_append {
				stack.push(item.clone());
			}
		}
	}

	(dfs_order_to_block, id_to_dfs_order)
}
