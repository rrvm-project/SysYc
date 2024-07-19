use std::collections::HashSet;

enum CallingStack {
	Prepare(usize),
	LoopTree(usize, usize), //low[u]=min(low[u],low[v])
	Back(usize),
}

pub struct Tarjan {
	dfs: Vec<Option<usize>>,
	low: Vec<Option<usize>>,
	in_stack: Vec<bool>,
	object_stack: Vec<usize>,
	calling_stack: Vec<CallingStack>,
	next_dfs_order: usize,
}

pub trait Graph<'a> {
	fn next(&'a self, u: usize) -> Box<dyn Iterator<Item = usize> + 'a>;
}

impl Tarjan {
	pub fn new(size: usize) -> Self {
		let mut nones = Vec::with_capacity(size);
		nones.resize(size, None);

		let mut falses = Vec::with_capacity(size);
		falses.resize(size, false);

		Self {
			dfs: nones.clone(),
			low: nones,
			in_stack: falses,
			calling_stack: vec![],
			object_stack: vec![],
			next_dfs_order: 0,
		}
	}

	pub fn work<'a>(
		mut self,
		graph: &'a (impl Graph<'a> + 'a),
	) -> Vec<Vec<usize>> {
		if self.dfs.is_empty() {
			return vec![];
		}
		let mut sccs = vec![];
		let mut node_to_scc = vec![];
		node_to_scc.resize(self.dfs.len(), None);

		for item in 0..self.dfs.len() {
			self.calling_stack.push(CallingStack::Prepare(item));
			// println!("{}:{:?}", item, graph.next(item).collect::<Vec<_>>());
		}
		self.main_loop(graph, &mut sccs, &mut node_to_scc);

		let mut solved = HashSet::new();
		for scc in &sccs {
			for item in scc {
				solved.insert(*item);
			}
			for item in scc {
				for need in graph.next(*item) {
					if !solved.contains(&need) {
						unreachable!("{} needs {}", item, need);
					}
				}
			}
		}
		sccs
	}

	fn main_loop<'a>(
		&mut self,
		graph: &'a (impl Graph<'a> + 'a),
		result: &mut Vec<Vec<usize>>,
		node_to_scc: &mut [Option<usize>],
	) {
		fn min(a: Option<usize>, b: Option<usize>) -> usize {
			let a = a.unwrap();
			let b = b.unwrap();
			a.min(b)
		}

		while let Some(order) = self.calling_stack.pop() {
			match order {
				CallingStack::Prepare(u) => {
					if node_to_scc[u].is_some() {
						continue;
					}
					self.dfs[u] = Some(self.next_dfs_order);
					self.low[u] = Some(self.next_dfs_order);
					self.next_dfs_order += 1;
					self.in_stack[u] = true;
					self.object_stack.push(u);
					self.calling_stack.push(CallingStack::Back(u));
					for v in graph.next(u) {
						if node_to_scc[v].is_some() {
							continue;
						}
						if self.dfs[v].is_none() {
							//Tree edge
							self.calling_stack.push(CallingStack::LoopTree(u, v));
							self.calling_stack.push(CallingStack::Prepare(v));
						} else if self.in_stack[v] {
							// Back edge
							self.low[u] = Some(min(self.low[u], self.dfs[v]));
						}
					}
				}
				CallingStack::LoopTree(u, v) => {
					self.low[u] = Some(min(self.low[u], self.low[v]));
				}
				CallingStack::Back(u) => {
					if self.dfs[u] == self.low[u] {
						let mut scc = vec![];
						loop {
							let this = self.object_stack.pop().unwrap();
							self.in_stack[u] = false;
							scc.push(this);
							node_to_scc[this] = Some(result.len());
							if this == u {
								break;
							}
						}
						result.push(scc);
					}
				}
			}
		}

		assert!(self.object_stack.is_empty());
	}
}
