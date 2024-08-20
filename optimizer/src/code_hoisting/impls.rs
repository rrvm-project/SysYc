use std::collections::{HashMap, HashSet};

use llvm::{LlvmInstrVariant::*, LlvmTemp, Value};
use rrvm::{
	dominator::LlvmDomTree,
	program::{LlvmFunc, LlvmProgram},
	LlvmNode,
};

use crate::{
	metadata::{FuncData, MetaData},
	RrvmOptimizer,
};

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

struct Solver<'a> {
	changed: bool,
	dom_tree: LlvmDomTree,
	stack: Vec<NodeInfo>,
	func_data: &'a mut FuncData,
	temp_mapper: HashMap<LlvmTemp, Value>,
}

impl<'a> Solver<'a> {
	pub fn new(func: &LlvmFunc, metadata: &'a mut MetaData) -> Self {
		Self {
			dom_tree: LlvmDomTree::new(&func.cfg, false),
			stack: Vec::new(),
			changed: false,
			func_data: metadata.get_func_data(&func.name),
			temp_mapper: HashMap::new(),
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

	fn hoist(&mut self, node: &LlvmNode) {
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
				self.changed = true;
				false
			} else {
				true
			}
		});
	}

	pub fn dfs(&mut self, node: LlvmNode) {
		let children = self.dom_tree.get_children(node.borrow().id).clone();
		let info = self.get_info(&node);
		self.hoist(&node);
		self.stack.push(info);
		for v in children {
			self.dfs(v);
		}
		self.stack.pop();
	}

	// hoist instructions which existed in all successors
	pub fn hoist_common(&mut self, node: &LlvmNode) {
		let children = self.dom_tree.get_children(node.borrow().id).clone();
		for v in children.iter() {
			self.hoist_common(v);
		}
		if node.borrow().succ.len() < 2 {
			return;
		};
		if node
			.borrow()
			.succ
			.iter()
			.any(|v| !self.dom_tree.dominates(node.borrow().id, v.borrow().id))
		{
			return;
		}
		if node.borrow().succ.iter().any(|v| v.borrow().prev.len() != 1) {
			return;
		}
		let mut common = node.borrow().succ[0].borrow().instrs.clone();
		for child in node.borrow().succ.iter() {
			let length = common
				.iter()
				.zip(child.borrow().instrs.iter())
				.take_while(|(a, b)| self.func_data.is_equal(a, b))
				.count();
			common.truncate(length);
		}
		for succ in node.borrow().succ.iter() {
			let instrs: Vec<_> =
				succ.borrow_mut().instrs.drain(0..common.len()).collect();
			for (instr, common_instr) in instrs.iter().zip(common.iter()) {
				if let (Some(x), Some(y)) =
					(instr.get_write(), common_instr.get_write())
				{
					self.temp_mapper.insert(x, y.into());
				}
			}
		}
		self.changed |= !common.is_empty();
		node.borrow_mut().instrs.extend(common);
	}

	pub fn map_temp(&self, func: &LlvmFunc) {
		for node in func.cfg.blocks.iter() {
			let block = &mut node.borrow_mut();
			for instr in block.instrs.iter_mut() {
				instr.map_temp(&self.temp_mapper);
			}
			for instr in block.phi_instrs.iter_mut() {
				for (value, _) in instr.source.iter_mut() {
					value.map_temp(&self.temp_mapper);
				}
			}
			if let Some(instr) = block.jump_instr.as_mut() {
				instr.map_temp(&self.temp_mapper)
			}
		}
	}
}

impl RrvmOptimizer for CodeHoisting {
	fn new() -> Self {
		Self {}
	}

	fn apply(
		self,
		program: &mut LlvmProgram,
		metadata: &mut MetaData,
	) -> Result<bool> {
		fn solve(func: &LlvmFunc, metadata: &mut MetaData) -> bool {
			let mut solver = Solver::new(func, metadata);
			solver.dfs(func.cfg.get_entry().clone());
			solver.hoist_common(&func.cfg.get_entry());
			solver.map_temp(func);
			solver.changed
		}

		Ok(
			program
				.funcs
				.iter()
				.fold(false, |last, func| solve(func, metadata) || last),
		)
	}
}
