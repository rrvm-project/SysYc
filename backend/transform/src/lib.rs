use instr_dag::InstrDag;
use instruction::{riscv::riscvinstr::RiscvInstr, temp::TempManager};
use llvm::LlvmInstr;
use rrvm::{basicblock::Node, program::*, RiscvCFG};
use utils::errors::Result;

pub mod instr_dag;
pub mod transformer;

pub fn convert_func(func: LlvmFunc) -> Result<RiscvFunc> {
	let mut blocks = Vec::new();
	let mgr = &mut TempManager::new();
	for v in func.cfg.blocks {
		blocks.push(transform_basicblock(v, mgr)?)
	}
	let cfg = RiscvCFG { blocks };
	Ok(RiscvFunc {
		cfg,
		name: func.name,
		params: func.params,
		ret_type: func.ret_type,
	})
}

pub fn transform_basicblock(
	block: Node<LlvmInstr>,
	mgr: &mut TempManager,
) -> Result<Node<RiscvInstr>> {
	let _instr_dag = InstrDag::new(&block.borrow().instrs, mgr)?;
	// let v = BasicBlock::new();
	// {
	// 	instrs: instr_serialize(instr_dag)?,
	// 	..block
	// }
	todo!()
}
