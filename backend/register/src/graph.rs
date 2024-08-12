use std::{
	cmp::max,
	collections::{BinaryHeap, HashMap, HashSet},
	hash::Hash,
};

use utils::union_find::UnionFind;

use crate::utils::{priority, FindAvailable};

pub struct InterferenceGraph<T: Hash + Eq + Copy, U> {
	allocator: Box<dyn FindAvailable<U>>,
	uf: UnionFind<T>,
	weights: HashMap<T, f64>,
	colors: HashMap<T, U>,
	edges: HashMap<T, HashSet<T>>,
	merge_benefit: HashMap<(T, T), f64>, // HACK: benefit varies while coalescing ?
}

impl<T, U> InterferenceGraph<T, U>
where
	T: Hash + Eq + Copy,
	U: PartialEq + Eq + Copy + Hash,
{
	pub fn add_edge(&mut self, x: T, y: T) {
		if x != y {
			self.edges.entry(x).or_default().insert(y);
			self.edges.entry(y).or_default().insert(x);
		}
	}
	pub fn add_benefit(&mut self, x: &T, y: &T, benefit: f64) {
		*self.merge_benefit.entry((*x, *y)).or_default() += benefit;
		*self.merge_benefit.entry((*y, *x)).or_default() += benefit;
	}
	pub fn add_weight(&mut self, x: T, w: f64) {
		*self.weights.entry(x).or_default() += w;
	}
	pub fn set_color(&mut self, x: &T, col: U) {
		self.colors.insert(*x, col);
	}
	fn can_set(&self, x: &T, reg: U) -> bool {
		self.edges.get(x).map_or(true, |arr| {
			arr.iter().all(|v| self.colors.get(v).map_or(true, |v| *v != reg))
		})
	}
	fn get_nodes(&mut self) -> Vec<T> {
		self.weights.keys().copied().filter(|v| self.uf.is_root(*v)).collect()
	}
	fn get_degree(&self, x: &T) -> usize {
		self.edges.get(x).map(|v| v.len()).unwrap_or_default()
	}
	fn get_weight(&self, x: &T) -> f64 {
		self.weights.get(x).copied().unwrap_or(0.into())
	}
	fn get_priority(&self, x: &T) -> f64 {
		priority(self.get_weight(x), self.get_degree(x))
	}
	fn can_merge(&self, x: &T, y: &T) -> bool {
		x != y
			&& self.edges.get(x).map_or(true, |s| !s.contains(y))
			&& match (self.colors.get(x), self.colors.get(y)) {
				(Some(x), Some(y)) => x == y,
				(Some(x), None) => self.can_set(y, *x),
				(None, Some(y)) => self.can_set(x, *y),
				_ => true,
			}
	}
	fn briggs_cond(&self, x: &T, y: &T) -> bool {
		let deg = self
			.edges
			.get(x)
			.unwrap_or(&HashSet::new())
			.union(self.edges.get(y).unwrap_or(&HashSet::new()))
			.collect::<HashSet<_>>()
			.len();
		deg
			< max(
				self.allocator.len(),
				max(self.get_degree(x), self.get_degree(y)),
			)
	}
	fn merge(&mut self, x: &T, y: &T) {
		self.uf.merge(*y, *x);
		if let Some(color) = self.colors.remove(y) {
			self.colors.insert(*x, color);
		}
		if let Some(edges) = self.edges.remove(y) {
			edges.iter().for_each(|v| {
				let entry = self.edges.entry(*v).or_default();
				entry.insert(*x);
				entry.remove(y);
			});
			self.edges.entry(*x).or_default().extend(edges);
		}
	}

	pub fn new(allocator: Box<dyn FindAvailable<U>>) -> Self {
		Self {
			allocator,
			uf: UnionFind::<T>::default(),
			weights: HashMap::new(),
			colors: HashMap::new(),
			edges: HashMap::new(),
			merge_benefit: HashMap::new(),
		}
	}

	pub fn coloring(&mut self) -> HashSet<T> {
		let mut nodes: Vec<_> =
			self.get_nodes().iter().map(|v| (*v, self.get_priority(v))).collect();
		nodes.sort_by(|(_, x), (_, y)| x.total_cmp(y));
		nodes
			.into_iter()
			.filter_map(|(node, _)| {
				let neighbors = self.edges.remove(&node).unwrap_or_default();
				let used: HashSet<_> = neighbors
					.into_iter()
					.filter_map(|u| self.colors.get(&u).copied())
					.collect();
				if let std::collections::hash_map::Entry::Vacant(e) =
					self.colors.entry(node)
				{
					if let Some(reg) = self.allocator.find_available(&used) {
						e.insert(reg);
					} else {
						return Some(node);
					}
				}
				None
			})
			.collect()
	}

	pub fn get_map(mut self) -> HashMap<T, U> {
		self
			.weights
			.into_keys()
			.map(|v| (v, *self.colors.get(&self.uf.find(v)).unwrap()))
			.collect()
	}
	pub fn get_colors(&self) -> usize {
		self.allocator.len()
	}
}

impl<T, U> InterferenceGraph<T, U>
where
	T: Hash + Eq + Copy + Ord,
	U: PartialEq + Eq + Copy + Hash,
{
	pub fn eliminate_move(&mut self) {
		let edges = std::mem::take(&mut self.merge_benefit);
		let mut edges = edges.into_iter().collect::<Vec<_>>();
		edges.sort_by(|(_, x), (_, y)| x.total_cmp(y));
		loop {
			let mut flag = true;
			for ((u, v), _) in edges.iter_mut() {
				*u = self.uf.find(*u);
				*v = self.uf.find(*v);
				if self.can_merge(u, v) {
					flag = false;
					self.merge(u, v);
					*v = *u;
				}
			}
			edges.retain(|((x, y), _)| {
				x != y
					&& match (self.colors.get(x), self.colors.get(y)) {
						(Some(x), Some(y)) => x == y,
						_ => true,
					}
			});
			if flag {
				break;
			}
		}
	}

	pub fn coalescing(&mut self) {
		let mut heap: BinaryHeap<_> =
			self.get_nodes().into_iter().map(|v| (self.get_degree(&v), v)).collect();
		while let Some((d, x)) = heap.pop() {
			if d > self.allocator.len() {
				break;
			}
			for (_, y) in heap.iter().take(self.allocator.len().pow(2)) {
				if self.can_merge(&x, y) && self.briggs_cond(&x, y) {
					self.merge(y, &x);
					break;
				}
			}
		}
	}
}
