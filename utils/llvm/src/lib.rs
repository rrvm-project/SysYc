pub mod func;
pub mod label;
pub mod llvmfuncemitter;
pub mod llvminstr;
pub mod llvmop;
pub mod llvmvar;
pub mod parser;
pub mod temp;

mod impls;
mod utils_llvm;

use func::LlvmFunc;
use std::{collections::HashMap, fmt::Display};

use utils::InitValueItem;

#[allow(unused)]
pub struct LlvmProgram {
	pub funcs: Vec<LlvmFunc>,
	pub global_vars: HashMap<String, Vec<InitValueItem>>,
}

impl Display for LlvmProgram {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		writeln!(f, "Global Vars:")?;
		for global_var in &self.global_vars {
			writeln!(f, "{:?}", global_var)?;
		}
		writeln!(f)?;
		for func in &self.funcs {
			writeln!(f, "{}", func)?;
		}
		Ok(())
	}
}
