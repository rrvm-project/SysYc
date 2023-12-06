use std::collections::HashMap;

use crate::temp::Temp;

use super::{reg::RiscvReg, value::RiscvTemp};

pub fn unwarp_temp(temp: &RiscvTemp) -> Option<Temp> {
	match temp {
		RiscvTemp::VirtReg(v) => Some(*v),
		_ => None,
	}
}

pub fn unwarp_temps(temp: Vec<&RiscvTemp>) -> Vec<Temp> {
	temp.into_iter().filter_map(unwarp_temp).collect()
}

pub fn map_temp(temp: &mut RiscvTemp, map: &HashMap<Temp, RiscvReg>) {
	let reg = match temp {
		RiscvTemp::VirtReg(v) => map.get(v).unwrap(),
		RiscvTemp::PhysReg(v) => v,
	};
	*temp = RiscvTemp::PhysReg(*reg);
}
