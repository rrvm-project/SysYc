use std::collections::HashMap;

use basicblock::BasicBlock;
use instruction::{
	instr_dag::InstrDag, instr_schedule::instr_serialize, InstrSet,
};
use llvm::func::LlvmFunc;
use utils::errors::Result;

pub mod basicblock;

pub fn transform_basicblock(block: BasicBlock) -> Result<BasicBlock> {
	let mut instr_dag = InstrDag::new(block.instrs);
	instr_dag.convert()?;
	Ok(BasicBlock {
		instrs: instr_serialize(instr_dag)?,
		..block
	})
}

pub fn build_from(func: LlvmFunc) -> Vec<BasicBlock> {
	let mut cur_id = 0;
	let mut cur_label = None;
	let mut cur_instr_set = Vec::new();
	let mut result = Vec::new();
	let mut label2id = HashMap::new();

	{
		let mut empty = false;
		for instr in func.body.iter() {
			if let Some(label) = instr.get_label() {
				if !empty {
					cur_id += 1;
					empty = true;
				}
				label2id.insert(label, cur_id);
			} else if instr.is_seq() {
				empty = false
			} else {
				cur_id += 1;
				empty = true;
			}
		}
	}
	for instr in func.body.into_iter() {
		if let Some(label) = instr.get_label() {
			if !cur_instr_set.is_empty() {
				result.push(BasicBlock::new(
					cur_id,
					cur_label,
					InstrSet::LlvmInstrSet(cur_instr_set),
				));
				cur_id += 1;
				cur_instr_set = Vec::new();
			}
			cur_label = Some(label);
		} else if instr.is_seq() {
			cur_instr_set.push(instr);
		} else {
			let mut basicblock = BasicBlock::new(
				cur_id,
				cur_label,
				InstrSet::LlvmInstrSet(cur_instr_set),
			);
			basicblock.succ = instr
				.get_succ()
				.into_iter()
				.map(|v| *label2id.get(&v).expect("邪了门了，怎么会找不到 label"))
				.collect();
			result.push(basicblock);
			cur_id += 1;
			cur_label = None;
			cur_instr_set = Vec::new();
		}
	}
	result
}
