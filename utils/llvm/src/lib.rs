pub mod func;
pub mod label;
pub mod llvmfuncemitter;
pub mod llvminstr;
pub mod llvmop;
pub mod llvmvar;
pub mod parser;
pub mod temp;

mod impls;
mod utils;

use func::LlvmFunc;
use llvminstr::GlobalVar;
use std::fmt::Display;

#[allow(unused)]
pub struct LlvmProgram {
	pub funcs: Vec<LlvmFunc>,
	pub global_vars: Vec<GlobalVar>,
}

impl Display for LlvmProgram {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		// TODO: 暂时不打印全局变量
		// for global_var in &self.global_vars {
		// 	writeln!(f, "{}", global_var)?;
		// }
		for func in &self.funcs {
			writeln!(f, "{}", func)?;
		}
		Ok(())
	}
}
