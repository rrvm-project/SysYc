use basicblock::BasicBlock;
use llvm::func::LlvmFunc;

pub mod basicblock;

#[allow(unused)]
pub fn build_from(func: LlvmFunc) -> Vec<BasicBlock> {
	todo!()
}
