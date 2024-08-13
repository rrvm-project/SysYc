use std::collections::HashSet;

use llvm::{LlvmInstrVariant::*, LlvmTemp};
use rrvm::{
	dominator::LlvmDomTree,
	program::{LlvmFunc, LlvmProgram},
	LlvmNode,
};

use crate::{metadata::MetaData, RrvmOptimizer};

use super::CodeHoisting;

use utils::Result;

#[derive(Clone)]
struct NodeInfo {
	node: LlvmNode,
	anti_temps: HashSet<LlvmTemp>,
}

impl NodeInfo {
	pub fn insert(&mut self, temp: LlvmTemp) {
		self.anti_temps.insert(temp);
	}
	pub fn contains(&self, temp: &LlvmTemp) -> bool {
		self.anti_temps.contains(temp)
	}
}

struct Solver {
	dom_tree: LlvmDomTree,
	stack: Vec<NodeInfo>,
}

impl Solver {
	pub fn new(func: &LlvmFunc) -> Self {
		Self {
			dom_tree: LlvmDomTree::new(&func.cfg, false),
			stack: Vec::new(),
		}
	}

	fn get_info(&mut self, node: &LlvmNode) -> NodeInfo {
		let mut info = NodeInfo {
			node: node.clone(),
			anti_temps: self
				.stack
				.last()
				.map(|v| v.anti_temps.clone())
				.unwrap_or_default(),
		};
		for instr in node.borrow().phi_instrs.iter() {
			info.insert(instr.target.clone());
		}
		for instr in node.borrow().instrs.iter() {
			if let Some(target) = instr.get_write() {
				info.insert(target);
			}
		}
		info
	}

	fn hoist(&mut self, node: &LlvmNode) -> bool {
		let mut flag = false;
		let node_weight = node.borrow().weight;
		node.borrow_mut().instrs.retain(|instr| {
			let mut best_node: Option<LlvmNode> = None;
			let mut best_weight = node_weight * 0.99;
			for info in self.stack.iter_mut().rev() {
				match instr.get_variant() {
					LoadInstr(instr) if !instr.addr.unwrap_temp().unwrap().is_global => {}
					StoreInstr(_) => {}
					CallInstr(_) => {}
					_ => {
						if instr
							.get_read()
							.iter()
							.all(|temp| info.contains(temp) || temp.is_global)
						{
							if info.node.borrow().weight < best_weight {
								best_weight = info.node.borrow().weight;
								best_node = Some(info.node.clone());
							}
						} else {
							break;
						}
					}
				}
			}
			if let Some(best_node) = best_node {
				best_node.borrow_mut().instrs.push(instr.clone());
				if let Some(target) = instr.get_write() {
					for info in self.stack.iter_mut().rev() {
						info.insert(target.clone());
						if std::ptr::eq(best_node.as_ptr(), info.node.as_ptr()) {
							break;
						}
					}
				}
				flag = true;
				false
			} else {
				true
			}
		});
		flag
	}

	pub fn dfs(&mut self, node: LlvmNode) -> bool {
		let children = self.dom_tree.get_children(node.borrow().id).clone();
		let info = self.get_info(&node);
		let mut flag = self.hoist(&node);
		self.stack.push(info);
		for v in children {
			flag |= self.dfs(v);
		}
		self.stack.pop();
		flag
	}
}

impl RrvmOptimizer for CodeHoisting {
	fn new() -> Self {
		Self {}
	}

	fn apply(
		self,
		program: &mut LlvmProgram,
		_metadata: &mut MetaData,
	) -> Result<bool> {
		fn solve(func: &LlvmFunc) -> bool {
			let mut solver = Solver::new(func);
			solver.dfs(func.cfg.get_entry().clone())
		}

		Ok(program.funcs.iter().fold(false, |last, func| solve(func) || last))
	}
}
