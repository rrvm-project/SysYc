use llvm::{CloneLlvmInstr, PhiInstr};
use rand::{rngs::StdRng, SeedableRng};
use rrvm::{
	dominator::DomTree,
	program::{LlvmFunc, LlvmProgram},
	LlvmNode,
};
use utils::{Label, Result};

use crate::{
	metadata::{FuncData, MetaData},
	number::Number,
	RrvmOptimizer,
};

use super::{
	utils::{work, NodeInfo},
	GlobalValueNumbering,
};

struct Solver<'a> {
	dom_tree: DomTree,
	rng: StdRng,
	stack: Vec<NodeInfo>,
	func_data: &'a mut FuncData,
}

impl<'a> Solver<'a> {
	pub fn new(func: &LlvmFunc, func_data: &'a mut FuncData) -> Self {
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
			func_data,
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
		self.func_data.num_mapper.extend(info.num_mapper.clone());
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
		fn solve(func: &LlvmFunc, func_data: &mut FuncData) -> bool {
			func_data.clear_num_mapper();
			let mut solver = Solver::new(func, func_data);
			solver.dfs(func.cfg.get_entry().clone())
		}

		Ok(program.funcs.iter().fold(false, |last, func| {
			solve(func, metadata.get_func_data(&func.name)) || last
		}))
	}
}
