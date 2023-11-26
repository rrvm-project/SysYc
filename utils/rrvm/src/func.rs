use llvm::llvmvar::VarType;

use crate::cfg::CFG;

pub struct RrvmFunc {
	pub cfg: CFG,
	pub name: String,
	pub params: Vec<(String, VarType)>,
}
