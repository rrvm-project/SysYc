use std::collections::{HashMap, HashSet};

use instruction::{riscv::reg::RiscvReg, Temp};
use utils::union_find::UnionFind;

pub fn priority(weight: f64, degree: usize) -> f64 {
	degree as f64 + 1f64 / weight
}

pub fn get_degree(
	x: &Temp,
	edges: &HashMap<Temp, Vec<Temp>>,
	union_find: &mut UnionFind<Temp>,
) -> usize {
	edges
		.get(x)
		.map_or(0, |a| a.iter().filter(|&&v| union_find.is_root(v)).count())
}

pub trait FindAvailable<T> {
	fn find_available(&mut self, x: &HashSet<T>) -> Option<T>;
	fn len(&self) -> usize;
	fn is_empty(&self) -> bool {
		self.len() == 0
	}
}

impl FindAvailable<RiscvReg> for &[RiscvReg] {
	fn find_available(&mut self, x: &HashSet<RiscvReg>) -> Option<RiscvReg> {
		self.iter().find(|v| !x.contains(v)).copied()
	}
	fn len(&self) -> usize {
		self.as_ref().len()
	}
}
