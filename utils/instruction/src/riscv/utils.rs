use std::collections::HashMap;

use utils::{mapper::LabelMapper, Label};

use crate::temp::Temp;

use super::{
	reg::RiscvReg,
	value::{RiscvImm, RiscvTemp},
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

pub fn map_temp(temp: &mut RiscvTemp, map: &HashMap<Temp, RiscvReg>) {
	let reg = match temp {
		RiscvTemp::VirtReg(v) => map.get(v).unwrap(),
		RiscvTemp::PhysReg(v) => v,
	};
	*temp = RiscvTemp::PhysReg(*reg);
}

pub fn map_label(label: &mut Label, map: &mut LabelMapper) {
	*label = map.get(label.clone())
}

pub fn map_imm_label(val: &mut RiscvImm, map: &mut LabelMapper) {
	if let RiscvImm::Label(label) = val {
		map_label(label, map)
	}
}
