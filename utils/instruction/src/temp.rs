use std::{cmp::Ordering, collections::HashMap, fmt::Display};

use utils::TempTrait;

use crate::riscv::{reg::RiscvReg, value::RiscvTemp};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Temp {
	pub id: i32,
	pub pre_color: Option<RiscvReg>,
}

impl PartialOrd for Temp {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.id.cmp(&other.id))
	}
}

impl Display for Temp {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "%{}", self.id)
	}
}

impl TempTrait for Temp {}

impl Temp {
	fn new(id: i32) -> Self {
		Self {
			id,
			pre_color: None,
		}
	}
}

pub struct TempManager {
	pub total: i32,
	pub total_pre_color: i32,
	llvm2riscv: HashMap<llvm::temp::Temp, RiscvTemp>,
}

impl TempManager {
	pub fn new(total: i32) -> Self {
		Self {
			total,
			total_pre_color: 0,
			llvm2riscv: HashMap::new(),
		}
	}
	pub fn new_temp(&mut self) -> RiscvTemp {
		self.total += 1;
		RiscvTemp::VirtReg(Temp::new(self.total))
	}
	pub fn new_raw_temp(&mut self, temp: &Temp) -> Temp {
		let id = if temp.pre_color.is_some() {
			self.total_pre_color -= 1;
			self.total_pre_color
		} else {
			self.total += 1;
			self.total
		};
		Temp {
			id,
			pre_color: temp.pre_color,
		}
	}
	pub fn new_pre_color_temp(&mut self, reg: RiscvReg) -> RiscvTemp {
		self.total_pre_color -= 1;
		RiscvTemp::VirtReg(Temp {
			id: self.total_pre_color,
			pre_color: Some(reg),
		})
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
