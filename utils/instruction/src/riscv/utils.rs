use std::collections::HashMap;

use utils::{mapper::LabelMapper, Label};

use crate::temp::Temp;

use super::{
	reg::RiscvReg,
	value::{RiscvImm, RiscvTemp},
	virt_mem::VirtAddr,
};

pub fn unwarp_temp(temp: RiscvTemp) -> Option<Temp> {
	match temp {
		RiscvTemp::VirtReg(v) => Some(v),
		_ => None,
	}
}

pub fn unwarp_temps(temp: Vec<RiscvTemp>) -> Vec<Temp> {
	temp.into_iter().filter_map(unwarp_temp).collect()
}

pub fn unwarp_imm(temp: &RiscvImm) -> Option<RiscvTemp> {
	match temp {
		RiscvImm::OffsetReg(_, v) => Some(*v),
		_ => None,
	}
}

pub fn unwarp_imms(temp: Vec<&RiscvImm>) -> Vec<RiscvTemp> {
	temp.into_iter().filter_map(unwarp_imm).collect()
}

pub fn map_temp(temp: &mut RiscvTemp, map: &HashMap<Temp, RiscvTemp>) {
	if let RiscvTemp::VirtReg(v) = temp {
		if let Some(new_temp) = map.get(v) {
			*temp = *new_temp;
		}
	}
}

pub fn map_virt_mem(
	rs: &mut RiscvImm,
	map: &HashMap<VirtAddr, (i32, RiscvTemp)>,
) {
	if let RiscvImm::VirtMem(mem) = rs {
		if let Some(addr) = map.get(mem) {
			*rs = (*addr).into()
		}
	}
}

pub fn map_imm_temp(val: &mut RiscvImm, map: &HashMap<Temp, RiscvTemp>) {
	if let RiscvImm::OffsetReg(_, temp) = val {
		map_temp(temp, map)
	}
}

pub fn map_label(label: &mut Label, map: &mut LabelMapper) {
	*label = map.get(label.clone())
}

pub fn map_imm_label(val: &mut RiscvImm, map: &mut LabelMapper) {
	if let RiscvImm::Label(label) = val {
		map_label(label, map)
	}
}

pub fn get_offset(index: usize) -> RiscvImm {
	(index as i32 * 8, RiscvReg::SP.into()).into()
}
#[derive(Default)]
pub struct PCRelMgr {
	pub cnt: usize,
	pub mapper: HashMap<(String, RiscvTemp), String>, // mapping function name and write temp to label
}

impl PCRelMgr {
	pub fn get_new_label(
		&mut self,
		func_name: String,
		temp: RiscvTemp,
	) -> String {
		let label = format!("PCRel_{}_{}", func_name, self.cnt);
		self.cnt += 1;
		self.mapper.insert((func_name, temp), label.clone());
		label
	}
	pub fn find_label(
		&self,
		func_name: &str,
		temp: &RiscvTemp,
	) -> Option<&String> {
		self.mapper.get(&(func_name.to_string(), *temp))
	}
}
