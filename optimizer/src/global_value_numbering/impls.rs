use llvm::CloneLlvmInstr;
use rand::{rngs::StdRng, SeedableRng};
use rrvm::{dominator::DomTree, program::LlvmFunc, LlvmNode};

use crate::RrvmOptimizer;

use super::{
	number::Number,
	utls::{work, NodeInfo},
	GlobalValueNumbering,
};

struct Solver {
	dom_tree: DomTree,
	rng: StdRng,
	stack: Vec<NodeInfo>,
}

impl Solver {
	pub fn new(func: &LlvmFunc) -> Self {
		let mut rng = StdRng::from_entropy();
		let mut info = NodeInfo::default();
		for param in func.params.iter() {
			let number = Number::new(&mut rng);
			info.set_number(param.unwrap_temp().unwrap(), number.clone());
			info.set_value(number, param.clone())
		}
		let stack = vec![info];
		Self {
			dom_tree: DomTree::new(&func.cfg, false),
			rng,
			stack,
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
			work(v.clone_box(), &mut info, &mut self.rng, &mut flag);
		});
		let instrs = std::mem::take(&mut block.instrs);
		block.instrs = instrs
			.into_iter()
			.filter_map(|v| work(v, &mut info, &mut self.rng, &mut flag))
			.collect();
		let new_jump = work(
			block.jump_instr.clone().unwrap(),
			&mut info,
			&mut self.rng,
			&mut flag,
		);
		block.set_jump(new_jump);
		let node_label = block.label();
		for v in block.succ.clone() {
			if std::ptr::eq(v.as_ptr(), node.as_ptr()) {
				for instr in block.phi_instrs.iter_mut() {
					for (value, label) in instr.source.iter_mut() {
						if *label == node_label {
							*value = info.map_value(value);
						}
					}
				}
			} else {
				for instr in v.borrow_mut().phi_instrs.iter_mut() {
					for (value, label) in instr.source.iter_mut() {
						if *label == node_label {
							*value = info.map_value(value);
						}
					}
				}
			}
		}
		(info, flag)
	}

	pub fn dfs(&mut self, node: LlvmNode) -> bool {
		let children = self.dom_tree.get_children(node.borrow().id).clone();
		let (info, mut flag) =
			self.get_info(node, self.stack.last().cloned().unwrap());
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
		program: &mut rrvm::prelude::LlvmProgram,
	) -> utils::Result<bool> {
		fn solve(func: &LlvmFunc) -> bool {
			let mut solver = Solver::new(func);
			solver.dfs(func.cfg.get_entry().clone())
		}

		Ok(program.funcs.iter().fold(false, |last, func| solve(func) || last))
	}
}
