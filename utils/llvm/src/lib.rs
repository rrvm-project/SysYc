pub mod basicblock;
pub mod cfg;
pub mod func;
pub mod llvmfuncemitter;
pub mod llvminstr;
pub mod llvmop;
pub mod llvmvar;
pub mod parser;
pub mod temp;

mod impls;
mod utils_llvm;

use std::{collections::HashMap, fmt::Display};

use func::LlvmFunc;
pub use llvminstr::*;
pub use temp::*;
use utils::InitValueItem;

#[allow(unused)]
pub struct LlvmProgram {
	pub funcs: Vec<LlvmFunc>,
	pub global_vars: HashMap<Temp, Vec<InitValueItem>>, // this is awful
}

impl Display for LlvmProgram {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		writeln!(f, "Global Vars:")?;
		for global_var in &self.global_vars {
			write!(f, "{} = global ", global_var.0)?;
			global_var.1.iter().for_each(|item| write!(f, "{} ", item).unwrap());
			writeln!(f)?;
		}
		writeln!(f)?;
		for func in &self.funcs {
			writeln!(f, "{}", func)?;
		}
		Ok(())
	}
}

pub enum LlvmInstrVariant<'a> {
	ArithInstr(&'a ArithInstr),
	LabelInstr(&'a LabelInstr),
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
