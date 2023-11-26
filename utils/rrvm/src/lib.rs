use basicblock::BasicBlock;
use instruction::{instr_dag::InstrDag, instr_schedule::instr_serialize};
use utils::errors::Result;

pub mod basicblock;
pub mod cfg;
pub mod func;

pub fn transform_basicblock(block: BasicBlock) -> Result<BasicBlock> {
	let mut instr_dag = InstrDag::new(block.instrs);
	instr_dag.convert()?;
	Ok(BasicBlock {
		instrs: instr_serialize(instr_dag)?,
		..block
	})
}
