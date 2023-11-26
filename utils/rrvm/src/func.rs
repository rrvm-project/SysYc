use llvm::Temp;

use crate::cfg::CFG;

pub struct RrvmFunc {
	pub cfg: CFG,
	pub name: String,
	pub params: Vec<Temp>,
}

impl RrvmFunc {
	pub fn new(cfg: CFG, name: String, params: Vec<Temp>) -> Self {
		Self { cfg, name, params }
	}
}
