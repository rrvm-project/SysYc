use std::{cmp::Ordering, collections::HashMap, fmt::Display};

use llvm::LlvmTemp;
use utils::TempTrait;

use crate::riscv::{reg::RiscvReg, value::RiscvTemp};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum VarType {
	Int,
	Float,
}

impl From<llvm::VarType> for VarType {
	fn from(var_type: llvm::VarType) -> Self {
		match var_type {
			llvm::VarType::F32 => VarType::Float,
			llvm::VarType::Void => unreachable!(),
			_ => VarType::Int,
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Temp {
	pub var_type: VarType,
	pub id: i32,
	pub pre_color: Option<RiscvReg>,
}

impl PartialOrd for Temp {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.id.cmp(&other.id))
	}
}

impl Ord for Temp {
	fn cmp(&self, other: &Self) -> Ordering {
		self.id.cmp(&other.id)
	}
}

impl Display for Temp {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self.pre_color {
			None => write!(f, "%{}", self.id),
			Some(v) => write!(f, "%{}({})", self.id, v),
		}
	}
}

impl TempTrait for Temp {}

impl Temp {
	fn new(id: i32, var_type: VarType) -> Self {
		Self {
			var_type,
			id,
			pre_color: None,
		}
	}
}

#[derive(Default)]
pub struct TempManager {
	pub total: i32,
	pub total_pre_color: i32,
	llvm2riscv: HashMap<LlvmTemp, RiscvTemp>,
}

impl TempManager {
	pub fn new_temp(&mut self, var_type: VarType) -> RiscvTemp {
		self.total += 1;
		RiscvTemp::VirtReg(Temp::new(self.total, var_type))
	}
	pub fn new_raw_temp(
		&mut self,
		temp: &Temp,
		flag: bool,
		var_type: VarType,
	) -> Temp {
		if temp.pre_color.is_some() && flag {
			self.total_pre_color -= 1;
			Temp {
				var_type,
				id: self.total_pre_color,
				pre_color: temp.pre_color,
			}
		} else {
			self.total += 1;
			Temp {
				var_type,
				id: self.total,
				pre_color: None,
			}
		}
	}
	pub fn new_pre_color_temp(&mut self, reg: RiscvReg) -> RiscvTemp {
		self.total_pre_color -= 1;
		RiscvTemp::VirtReg(Temp {
			var_type: reg.get_type(),
			id: self.total_pre_color,
			pre_color: Some(reg),
		})
	}
	pub fn get(&mut self, temp: &LlvmTemp) -> RiscvTemp {
		if let Some(v) = self.llvm2riscv.get(temp) {
			*v
		} else {
			let new = self.new_temp(temp.var_type.into());
			self.llvm2riscv.insert(temp.clone(), new);
			new
		}
	}
	pub fn get_pre_color(&mut self, temp: &LlvmTemp, reg: RiscvReg) -> RiscvTemp {
		if let Some(v) = self.llvm2riscv.get(temp) {
			*v
		} else {
			let new = self.new_pre_color_temp(reg);
			self.llvm2riscv.insert(temp.clone(), new);
			new
		}
	}
}
