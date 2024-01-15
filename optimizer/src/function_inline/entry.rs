use std::collections::{HashMap, HashSet};

use llvm::{LlvmTemp, VarType};
use rrvm::{basicblock::LlvmBasicBlock, program::LlvmProgram};

#[derive(Default)]
pub struct FuncEntry {
	pub var_type: VarType,
	pub params: Vec<LlvmTemp>,
	pub edges: Vec<(i32, i32)>,
	pub nodes: Vec<LlvmBasicBlock>,
}

pub fn get_func_table(
	func_list: HashSet<String>,
	program: &LlvmProgram,
) -> HashMap<String, FuncEntry> {
	let mut table = HashMap::new();
	for func in program.funcs.iter().filter(|v| func_list.contains(&v.name)) {
		let mut entry = FuncEntry {
			params: func.params.iter().filter_map(|v| v.unwrap_temp()).collect(),
			var_type: func.ret_type,
			..Default::default()
		};
		for block in func.cfg.blocks.iter() {
			let id = block.borrow().id;
			for v in block.borrow().succ.iter() {
				entry.edges.push((id, v.borrow().id));
			}
			entry.nodes.push(block.borrow().clone());
		}
		table.insert(func.name.clone(), entry);
	}
	table
}
