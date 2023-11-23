use crate::{context::IRPassContext, irpass::IRPass};
use llvm::{
	cfg::CFG,
	llvmop::{/*ArithOp, LlvmOp,*/ Value},
	LlvmProgram,
};
use std::collections::{HashMap, HashSet};
pub struct Svn {
	next_value_number: usize,
}

impl Svn {
	pub fn new() -> Self {
		Svn {
			next_value_number: 0,
		}
	}
}

impl Default for Svn {
	fn default() -> Self {
		Self::new()
	}
}

impl IRPass for Svn {
	fn pass(&mut self, program: &mut LlvmProgram, context: &mut IRPassContext) {
		for item in &mut program.funcs {
			let cfg = &mut item.cfg;
			println!("\n\nfor function {}", item.label);
			self.traverse_cfg(cfg, context);
		}
	}
}
#[allow(dead_code)]
enum LvnValueItem {
	LLValue(Value),
	Exp((String, Vec<Value>)),
}
#[allow(dead_code)]
struct BasicBlockLvnData {
	pub id: usize,
	pub value_to_number: HashMap<LvnValueItem, usize>,
	pub number_repr_value: HashMap<usize, Value>,
}

impl BasicBlockLvnData {
	fn new(id: usize) -> Self {
		BasicBlockLvnData {
			id,
			value_to_number: HashMap::new(),
			number_repr_value: HashMap::new(),
		}
	}
}

impl Svn {
	fn traverse_cfg(&mut self, cfg: &mut CFG, ctx: &mut IRPassContext) {
		let mut work_list = vec![cfg.entry];
		let mut visited = HashSet::<usize>::new();

		let mut id;
		let mut lvn_data: Vec<BasicBlockLvnData> = vec![];

		while !work_list.is_empty() {
			id = work_list.pop().unwrap();
			self.svn(id, cfg, &mut work_list, &mut lvn_data, &mut visited, ctx);
		}
	}

	fn svn(
		&mut self,
		id: usize,
		cfg: &mut CFG,
		work_list: &mut Vec<usize>,
		lvn_data: &mut Vec<BasicBlockLvnData>,
		visited: &mut HashSet<usize>,
		ctx: &mut IRPassContext,
	) {
		lvn_data.push(BasicBlockLvnData::new(id));

		visited.insert(id);
		self.lvn(id, cfg, lvn_data, ctx);

		let mut to_visit_next = vec![];

		for succ in &cfg.basic_blocks.get(&id).unwrap().succ {
			if cfg.basic_blocks.get(succ).unwrap().pred.len() == 1 {
				to_visit_next.push(*succ);
			} else if !visited.contains(succ) {
				work_list.push(*succ);
				println!("pushed into worklist {}", *succ);
			}
		}

		for item in to_visit_next {
			self.svn(item, cfg, work_list, lvn_data, visited, ctx);
		}

		lvn_data.pop();
	}

	#[allow(dead_code)]
	fn lvn(
		&mut self,
		id: usize,
		cfg: &mut CFG,
		_lvn_data: &mut [BasicBlockLvnData],
		_: &mut IRPassContext,
	) {
		if let Some(mut basicblock) = cfg.basic_blocks.remove(&id) {
			let mut _instrs = std::mem::take(&mut basicblock.instrs);

			let _ = self.next_value_number;

			// instrs.push(Box::new(llvm::ArithInstr{ target: Temp::new(114515, llvm::llvmvar::VarType::F32), op: ArithOp::Fadd, var_type: llvm::llvmvar::VarType::F32, lhs: Value::Float(0.3), rhs: Value::Float(7.66) }));

			basicblock.instrs = _instrs;
			cfg.basic_blocks.insert(id, basicblock);
		}
	}
}
