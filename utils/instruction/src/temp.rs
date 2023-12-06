use std::{collections::HashMap, fmt::Display};

use utils::TempTrait;

use crate::riscv::value::RiscvTemp;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Temp {
	pub id: u32,
}

impl Display for Temp {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "%{}", self.id)
	}
}

impl TempTrait for Temp {}

impl Temp {
	fn new(id: u32) -> Self {
		Self { id }
	}
}

pub struct TempManager {
	pub total: u32,
	llvm2riscv: HashMap<llvm::temp::Temp, RiscvTemp>,
}

impl TempManager {
	pub fn new(total: u32) -> Self {
		Self {
			total,
			llvm2riscv: HashMap::new(),
		}
	}
	pub fn new_temp(&mut self) -> RiscvTemp {
		self.total += 1;
		RiscvTemp::VirtReg(Temp::new(self.total))
	}
	pub fn get(&mut self, temp: &llvm::temp::Temp) -> RiscvTemp {
		if let Some(v) = self.llvm2riscv.get(temp) {
			*v
		} else {
			let new = self.new_temp();
			self.llvm2riscv.insert(temp.clone(), new);
			new
		}
	}
}
