use std::collections::HashMap;

use instruction::Temp;
use utils::union_find::UnionFind;

pub fn spill_cost(weight: f64, degree: usize) -> f64 {
	weight / degree as f64
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
