pub mod llvminstr;
pub mod llvmop;
pub mod llvmvar;
pub mod temp;

mod impls;
mod utils_llvm;

pub use llvminstr::*;
pub use llvmop::{Value, *};
pub use llvmvar::VarType;
pub use temp::*;

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
