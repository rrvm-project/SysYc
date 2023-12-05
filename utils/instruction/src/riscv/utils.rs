use crate::temp::Temp;

use super::value::RiscvTemp;

pub fn unwarp_temp(temp: &RiscvTemp) -> Option<Temp> {
	match temp {
		RiscvTemp::VirtReg(v) => Some(*v),
		_ => None,
	}
}

pub fn unwarp_temps(temp: Vec<&RiscvTemp>) -> Vec<Temp> {
	temp.into_iter().filter_map(unwarp_temp).collect()
}
