use std::collections::HashMap;

use llvm::{CloneLlvmInstr, LlvmTemp, PhiInstr};
use rand::{rngs::StdRng, SeedableRng};
use rrvm::{
	dominator::LlvmDomTree,
	program::{LlvmFunc, LlvmProgram},
	LlvmNode,
};
use utils::{Label, Result};

use crate::{metadata::MetaData, number::Number, RrvmOptimizer};

use super::{
	utils::{work, NodeInfo},
	GlobalValueNumbering,
};

struct Solver<'a> {
	dom_tree: LlvmDomTree,
	rng: StdRng,
	stack: Vec<NodeInfo>,
	metadata: &'a mut MetaData,
	num_mapper: HashMap<LlvmTemp, Number>,
}

impl<'a> Solver<'a> {
	pub fn new(func: &LlvmFunc, metadata: &'a mut MetaData) -> Self {
		let mut rng = StdRng::from_entropy();
		let mut info = NodeInfo::default();
		for param in func.params.iter() {
			let number = Number::new(&mut rng);
			info.set_number(param.unwrap_temp().unwrap(), number.clone());
			info.set_value(number, param.clone())
		}
		let stack = vec![info];
		Self {
			dom_tree: LlvmDomTree::new(&func.cfg, false),
			num_mapper: HashMap::new(),
			rng,
			stack,
			metadata,
		}
	}

	fn get_info(
		&mut self,
		node: LlvmNode,
		mut info: NodeInfo,
	) -> (NodeInfo, bool) {
		let mut flag = false;
		let mut block = node.borrow_mut();
		block.phi_instrs.iter().for_each(|v| {
			work(
				v.clone_box(),
				&mut info,
				&mut self.rng,
				&mut flag,
				self.metadata,
			);
		});
		let instrs = std::mem::take(&mut block.instrs);
		block.instrs = instrs
			.into_iter()
			.filter_map(|v| {
				work(v, &mut info, &mut self.rng, &mut flag, self.metadata)
			})
			.collect();
		let new_jump = work(
			block.jump_instr.clone().unwrap(),
			&mut info,
			&mut self.rng,
			&mut flag,
			self.metadata,
		);
		block.set_jump(new_jump);
		let node_label = block.label();
		fn map_value(instrs: &mut [PhiInstr], info: &NodeInfo, label: &Label) {
			for instr in instrs.iter_mut() {
				for (value, instr_label) in instr.source.iter_mut() {
					if label == instr_label {
						*value = info.map_value(value);
					}
				}
			}
		}
		for v in block.succ.clone() {
			if std::ptr::eq(v.as_ptr(), node.as_ptr()) {
				map_value(&mut block.phi_instrs, &info, &node_label)
			} else {
				map_value(&mut v.borrow_mut().phi_instrs, &info, &node_label)
			}
		}
		(info, flag)
	}

	pub fn dfs(&mut self, node: LlvmNode) -> bool {
		let children = self.dom_tree.get_children(node.borrow().id).clone();
		let (info, mut flag) =
			self.get_info(node, self.stack.last().cloned().unwrap());
		self.num_mapper.extend(info.num_mapper.clone());
		self.stack.push(info);
		for v in children {
			flag |= self.dfs(v);
		}
		self.stack.pop();
		flag
	}
}

impl RrvmOptimizer for GlobalValueNumbering {
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
			let flag = solver.dfs(func.cfg.get_entry().clone());
			metadata.get_func_data(&func.name).num_mapper = solver.num_mapper;
			flag
		}

		Ok(
			program
				.funcs
				.iter()
				.fold(false, |last, func| solve(func, metadata) || last),
		)
	}
}
