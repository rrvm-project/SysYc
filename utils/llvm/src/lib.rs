mod impls;
mod llvminstr;
mod llvmop;
mod llvmvar;
mod temp;
mod utils_llvm;

pub use impls::new_assign_instr;
pub use llvminstr::*;
pub use llvmop::{Value, *};
pub use llvmvar::*;
pub use temp::{LlvmTemp, LlvmTempManager};

pub enum LlvmInstrVariant<'a> {
	ArithInstr(&'a ArithInstr),
	CompInstr(&'a CompInstr),
	ConvertInstr(&'a ConvertInstr),
	JumpInstr(&'a JumpInstr),
	JumpCondInstr(&'a JumpCondInstr),
	PhiInstr(&'a PhiInstr),
	RetInstr(&'a RetInstr),
	AllocInstr(&'a AllocInstr),
	StoreInstr(&'a StoreInstr),
	LoadInstr(&'a LoadInstr),
	GEPInstr(&'a GEPInstr),
	CallInstr(&'a CallInstr),
}
