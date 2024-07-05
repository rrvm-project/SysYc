use std::collections::{HashMap, VecDeque};

use super::RangeAnalysis;
use crate::{
	range_analysis::block_imply::{
		add_implication, general_both, BlockImplyCondition,
	},
	RrvmOptimizer,
};
use llvm::{LlvmInstrTrait, LlvmTemp};
use rrvm::program::{LlvmFunc, LlvmProgram};
use utils::{errors::Result, from_label};

#[allow(clippy::type_complexity)]
fn process_function(func: &mut LlvmFunc) {
	func.cfg.analysis();

	let mut comparisons = HashMap::new();

	let mut block_condition: HashMap<
		i32,
		Vec<(i32, Option<LlvmTemp>, Option<LlvmTemp>)>,
	> = HashMap::new();

	let mut block_implies_workset =
		VecDeque::with_capacity(func.cfg.blocks.len());

	let mut id_to_block = HashMap::new();

	for block in func.cfg.blocks.iter() {
		block_implies_workset.push_back(block.borrow().id);
		id_to_block.insert(block.borrow().id, block.clone());

		// let v = block.borrow();
		//find all comparisons
		for instr in block.borrow().instrs.iter() {
			if let llvm::LlvmInstrVariant::CompInstr(i) = instr.get_variant() {
				// vec_comparison.push((i.target.clone(),i.lhs.clone(), i.rhs.clone(), i.op.clone(), block.borrow().id));
				comparisons.insert(
					i.target.clone(),
					(i.lhs.clone(), i.rhs.clone(), i.op, block.borrow().id),
				);
			}
		}

		if let Some(instr) = &block.borrow().jump_instr {
			match instr.get_variant() {
				llvm::LlvmInstrVariant::JumpCondInstr(jc) => {
					block_condition
						.entry(from_label(&jc.target_true))
						.or_default()
						.push((block.borrow().id, jc.cond.clone().into(), None));
					block_condition
						.entry(from_label(&jc.target_false))
						.or_default()
						.push((block.borrow().id, None, jc.cond.clone().into()));
				}
				llvm::LlvmInstrVariant::JumpInstr(j) => {
					block_condition
						.entry(from_label(&j.get_label()))
						.or_default()
						.push((block.borrow().id, None, None));
				}
				_ => {}
			}
		}
	}

	let mut block_implies: HashMap<i32, BlockImplyCondition> = HashMap::new();

	while let Some(current) = block_implies_workset.pop_front() {
		let current_entry =
			block_implies.entry(current).or_insert_with(BlockImplyCondition::new);
		let old_size = current_entry.size();

		if let Some(conditions) = block_condition.get(&current) {
			let new_cond = general_both(conditions.iter().map(|(p, pos, neg)| {
				let prev_entry =
					block_implies.entry(*p).or_insert_with(BlockImplyCondition::new);
				add_implication(prev_entry, pos.clone(), neg.clone())
			}));

			if old_size < new_cond.size() {
				if let Some(successor) = id_to_block.get(&current).map(|block| {
					block
						.borrow()
						.succ
						.iter()
						.map(|block| block.borrow().id)
						.collect::<Vec<i32>>()
				}) {
					for item in successor {
						block_implies_workset.push_back(item)
					}
				}
			}

			block_implies.insert(current, new_cond);
		}
	}

	// dbg!(&comparisons);
	// dbg!(&block_condition);
	dbg!(&block_implies);
}

impl RrvmOptimizer for RangeAnalysis {
	fn new() -> Self {
		Self {}
	}

	fn apply(self, program: &mut LlvmProgram) -> Result<bool> {
		program.funcs.iter_mut().for_each(process_function);
		Ok(false)
	}
}
