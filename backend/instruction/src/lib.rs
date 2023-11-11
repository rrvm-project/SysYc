mod instr_dag;
mod instr_schedule;
mod transformer;

use crate::instr_dag::InstrDag;
use basicblock::basicblock::BasicBlock;
use instr_schedule::instr_serialize;
use rrvm_func::rrvmfunc::RrvmFunc;

fn transform_basicblock(block: BasicBlock) -> BasicBlock {
	let mut instr_dag = InstrDag::new(block.instrs);
	instr_dag.convert();
	BasicBlock {
		instrs: instr_serialize(instr_dag),
		..block
	}
}

pub fn transform(mut func: RrvmFunc) -> RrvmFunc {
	func.cfg.basic_blocks =
		func.cfg.basic_blocks.into_iter().map(transform_basicblock).collect();
	func
}
