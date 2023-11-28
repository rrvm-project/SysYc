use basicblock::BasicBlock;
use cfg::CFG;
use instruction::riscv::riscvinstr::RiscvInstr;
use llvm::LlvmInstr;
use utils::errors::Result;

pub mod basicblock;
pub mod cfg;
pub mod func;
pub mod impls;
pub mod program;

pub type LlvmCFG = CFG<LlvmInstr>;
pub type RiscvCFG = CFG<RiscvInstr>;

pub fn transform_basicblock(
	_block: BasicBlock<LlvmInstr>,
) -> Result<BasicBlock<RiscvInstr>> {
	todo!()
	// let mut instr_dag = InstrDag::new(block.instrs);
	// instr_dag.convert()?;
	// Ok(BasicBlock {
	// 	instrs: instr_serialize(instr_dag)?,
	// 	..block
	// })
}
