use std::collections::{HashMap, HashSet};

use llvm::{LlvmTemp, Value};
use rrvm::{
	dominator::LlvmDomTree,
	program::{LlvmFunc, LlvmProgram},
	LlvmNode,
};

use llvm::LlvmInstrVariant::*;

use crate::metadata::{MetaData, VarIdent};

#[derive(Default)]
struct VarGraph {
	edges: HashMap<VarIdent, HashSet<VarIdent>>,
}

impl VarGraph {
	pub fn add_edge(&mut self, from: VarIdent, to: VarIdent) {
		self.edges.entry(from).or_default().insert(to);
	}
	pub fn get_neighbors(&self, ident: &VarIdent) -> HashSet<VarIdent> {
		self.edges.get(ident).cloned().unwrap_or_default()
	}
}

struct Solver<'a> {
	dom_tree: LlvmDomTree,
	ident_mapper: HashMap<LlvmTemp, VarIdent>,
	metadata: &'a mut MetaData,
	graph: &'a mut VarGraph,
}

impl<'a> Solver<'a> {
	pub fn new(
		func: &LlvmFunc,
		metadata: &'a mut MetaData,
		graph: &'a mut VarGraph,
	) -> Self {
		let mut ident_mapper = HashMap::new();
		for (index, param) in func.params.iter().enumerate() {
			if param.get_type().is_ptr() {
				ident_mapper
					.insert(param.unwrap_temp().unwrap(), (func.name.clone(), index));
			}
		}
		Self {
			dom_tree: LlvmDomTree::new(&func.cfg, false),
			ident_mapper,
			metadata,
			graph,
		}
	}
	pub fn dfs(&mut self, node: LlvmNode) {
		let block = &node.borrow();
		for instr in block.phi_instrs.iter() {
			if instr.var_type.is_ptr() {
				let src_addr = instr.source.iter().find_map(|(value, _)| {
					value.unwrap_temp().and_then(|v| self.ident_mapper.get(&v))
				});
				if let Some(ident) = src_addr {
					self.ident_mapper.insert(instr.target.clone(), ident.clone());
				}
			}
		}
		for instr in block.instrs.iter() {
			match instr.get_variant() {
				LoadInstr(instr) => {
					if let Value::Temp(addr) = &instr.addr {
						if addr.is_global {
							self
								.ident_mapper
								.insert(instr.target.clone(), (addr.name.clone(), 0));
						} else if let Some(ident) = self.ident_mapper.get(addr) {
							self.metadata.get_var_data(ident).to_load = true;
						}
					}
				}
				GEPInstr(instr) => {
					if let Value::Temp(addr) = &instr.addr {
						if let Some(ident) = self.ident_mapper.get(addr) {
							self.ident_mapper.insert(instr.target.clone(), ident.clone());
						}
					}
				}
				StoreInstr(instr) => {
					if let Value::Temp(addr) = &instr.addr {
						if let Some(ident) = self.ident_mapper.get(addr) {
							self.metadata.get_var_data(ident).to_store = true;
						}
					}
				}
				CallInstr(instr) => {
					for (index, (var_type, value)) in instr.params.iter().enumerate() {
						if var_type.is_ptr() {
							if let Value::Temp(temp) = value {
								if let Some(ident) = self.ident_mapper.get(temp) {
									self
										.graph
										.add_edge((instr.func.name.clone(), index), ident.clone());
								}
							}
						}
					}
				}
				_ => {}
			}
		}
		let children = self.dom_tree.get_children(block.id).clone();
		for v in children {
			self.dfs(v);
		}
	}
}

pub fn calc_var_data(program: &mut LlvmProgram, metadata: &mut MetaData) {
	let mut graph = VarGraph::default();
	for func in program.funcs.iter() {
		let mut solver = Solver::new(func, metadata, &mut graph);
		solver.dfs(func.cfg.get_entry());
	}
	let mut queue = Vec::new();
	for (ident, data) in metadata.var_data.iter() {
		if data.to_load {
			queue.push(ident.clone());
		}
	}
	while let Some(ident) = queue.pop() {
		for neighbor in graph.get_neighbors(&ident) {
			if metadata.get_var_data(&neighbor).to_load {
				continue;
			}
			metadata.get_var_data(&neighbor).to_load = true;
			queue.push(neighbor);
		}
	}
	let mut queue = Vec::new();
	for (ident, data) in metadata.var_data.iter() {
		if data.to_store {
			queue.push(ident.clone());
		}
	}
	while let Some(ident) = queue.pop() {
		for neighbor in graph.get_neighbors(&ident) {
			if metadata.get_var_data(&neighbor).to_store {
				continue;
			}
			metadata.get_var_data(&neighbor).to_store = true;
			queue.push(neighbor);
		}
	}
}
