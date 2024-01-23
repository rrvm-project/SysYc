use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::SolveTailRecursion;
use crate::RrvmOptimizer;
use llvm::{
	JumpInstr, LlvmInstrVariant, LlvmTempManager, PhiInstr, Value, VarType,
};
use rrvm::{
	basicblock::LlvmBasicBlock,
	cfg::link_node,
	program::{LlvmFunc, LlvmProgram},
};
use utils::{errors::Result, to_label};

impl RrvmOptimizer for SolveTailRecursion {
	fn new() -> Self {
		Self {}
	}
	fn apply(self, program: &mut LlvmProgram) -> Result<bool> {
		fn solve(func: &mut LlvmFunc, mgr: &mut LlvmTempManager) -> bool {
			if func.cfg.blocks.iter().any(|v| v.borrow().tail_call_func(&func.name)) {
				// prepare new temporaries
				let new_params: Vec<_> = func
					.params
					.iter()
					.map(|v| mgr.new_temp(v.get_type(), false))
					.collect();
				let value_mapper: HashMap<_, _> = func
					.params
					.iter()
					.zip(new_params.iter())
					.map(|(k, v)| (k.unwrap_temp().unwrap(), Value::Temp(v.clone())))
					.collect();

				// prepare new entry node and phi instructions to pass value
				let new_entry = LlvmBasicBlock::new(0, 1_f64);
				let mut sources: Vec<_> = func
					.params
					.iter()
					.map(|v| vec![(v.clone(), new_entry.label())])
					.collect();
				for block in func.cfg.blocks.iter() {
					let node = &mut block.borrow_mut();
					node.map_temp(&value_mapper);
				}
				func.total += 1;
				func.cfg.get_entry().borrow_mut().id = func.total;
				let succ = func.cfg.get_entry().borrow().succ.clone();
				for v in succ {
					for instr in v.borrow_mut().phi_instrs.iter_mut() {
						for (_, label) in instr.source.iter_mut() {
							if *label == to_label(0) {
								*label = to_label(func.total)
							}
						}
					}
				}

				// solve tail recursion
				for block in func.cfg.blocks.iter() {
					if block.borrow().tail_call_func(&func.name) {
						let succ = func.cfg.get_entry().clone();
						let node = &mut block.borrow_mut();
						let instr = node.instrs.pop().unwrap();
						if let LlvmInstrVariant::CallInstr(instr) = instr.get_variant() {
							for (src, param) in sources.iter_mut().zip(instr.params.iter()) {
								src.push((param.1.clone(), node.label()));
							}
						} else {
							unreachable!()
						}
						node.jump_instr = Some(JumpInstr::new(to_label(func.total)));
						node.succ.push(succ);
					}
				}

				// set phi instructions for the old entry
				func.cfg.get_entry().borrow_mut().phi_instrs = new_params
					.into_iter()
					.zip(sources)
					.map(|(target, source)| PhiInstr::new(target, source))
					.collect();

				// update the cfg
				let new_entry = Rc::new(RefCell::new(new_entry));
				link_node(&new_entry, &func.cfg.get_entry());
				new_entry.borrow_mut().gen_jump(VarType::Void);
				func.cfg.blocks.insert(0, new_entry);
				func.cfg.resolve_prev();
				true
			} else {
				false
			}
		}
		program.analysis();
		let flag = program.funcs.iter_mut().fold(false, |last, func| {
			solve(func, &mut program.temp_mgr) || last
		});
		Ok(flag)
	}
}
