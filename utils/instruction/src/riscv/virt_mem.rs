use std::collections::HashMap;

use super::value::RiscvTemp;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct VirtAddr {
	pub id: i32,

	/// offset from FP
	pub pre_color: Option<i32>,
}

#[derive(Default)]
pub struct VirtMemManager {
	total: i32,
	pub map: HashMap<RiscvTemp, VirtAddr>,
}

impl VirtMemManager {
	pub fn get_mem(&mut self, temp: RiscvTemp) -> VirtAddr {
		if let Some(addr) = self.map.get(&temp) {
			*addr
		} else {
			self.total += 1;
			let addr = VirtAddr {
				id: self.total,
				pre_color: None,
			};
			self.map.insert(temp, addr);
			addr
		}
	}

	pub fn new_mem_with_addr(&mut self, offset: i32) -> VirtAddr {
		self.total += 1;
		VirtAddr {
			id: self.total,
			pre_color: Some(offset),
		}
	}

	pub fn set_addr(&mut self, temp: RiscvTemp, addr: VirtAddr) {
		self.map.insert(temp, addr);
	}
}
