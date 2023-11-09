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

#[allow(unused)]
pub struct LlvmProgram {
	funcs: Vec<LlvmFunc>,
	global_vars: Vec<GlobalVar>,
}
