#![allow(unused)]

use std::collections::{HashMap, HashSet};

use llvm::{LlvmTemp, Value};
use rrvm::{
	dominator::LlvmDomTree,
	program::{LlvmFunc, LlvmProgram},
	LlvmNode,
};

use llvm::LlvmInstrVariant::*;

use crate::metadata::{MetaData, UsageInfo, VarIdent};

use super::var_data;

#[derive(Default)]
struct CallGraph {
	edges: HashMap<String, HashSet<String>>,
}

impl CallGraph {
	pub fn add_edge(&mut self, from: String, to: String) {
		self.edges.entry(from).or_default().insert(to);
	}
	pub fn get_neighbors(&self, func: &String) -> HashSet<String> {
		self.edges.get(func).cloned().unwrap_or_default()
	}
}

struct Solver<'a> {
	dom_tree: LlvmDomTree,
	temp_mapper: HashMap<LlvmTemp, String>,
	metadata: &'a mut MetaData,
	graph: &'a mut CallGraph,
	usage_info: UsageInfo,
}

impl<'a> Solver<'a> {
	pub fn new(
		func: &LlvmFunc,
		metadata: &'a mut MetaData,
		graph: &'a mut CallGraph,
	) -> Self {
		Self {
			dom_tree: LlvmDomTree::new(&func.cfg, false),
			temp_mapper: HashMap::new(),
			usage_info: UsageInfo::default(),
			metadata,
			graph,
		}
	}
	pub fn dfs(&mut self, node: LlvmNode) {
		let block = &mut node.borrow_mut();
		block.instrs.retain(|instr| {
			match instr.get_variant() {
				LoadInstr(instr) => {
					if let Value::Temp(addr) = &instr.addr {
						if addr.is_global {
							self.temp_mapper.insert(instr.target.clone(), addr.name.clone());
						} else if let Some(global_var) = self.temp_mapper.get(addr) {
							self.usage_info.may_loads.insert(global_var.clone());
						}
					}
				}
				GEPInstr(instr) => {
					if let Value::Temp(addr) = &instr.addr {
						if let Some(global_var) = self.temp_mapper.get(addr) {
							self.usage_info.may_loads.insert(global_var.clone());
						}
					}
				}
				StoreInstr(instr) => {
					if let Value::Temp(addr) = &instr.addr {
						if let Some(global_var) = self.temp_mapper.get(addr) {
							self.usage_info.may_stores.insert(global_var.clone());
							let var_data =
								self.metadata.get_var_data(&(global_var.clone(), 0));
							if !var_data.to_load {
								return false;
							}
						}
					}
				}
				CallInstr(instr) => {
					for (index, (var_type, value)) in instr.params.iter().enumerate() {
						if var_type.is_ptr() {
							if let Value::Temp(temp) = value {
								if let Some(global_var) = self.temp_mapper.get(temp) {
									let var_data = self
										.metadata
										.get_var_data(&(instr.func.name.clone(), index));
									if var_data.to_load {
										self.usage_info.may_loads.insert(global_var.clone());
									}
									if var_data.to_store {
										self.usage_info.may_stores.insert(global_var.clone());
									}
								}
							}
						}
					}
				}
				_ => {}
			}
			true
		});
		let children = self.dom_tree.get_children(block.id).clone();
		for v in children {
			self.dfs(v);
		}
	}
}

pub fn calc_func_data(program: &mut LlvmProgram, metadata: &mut MetaData) {
	let mut graph = CallGraph::default();
	for func in program.funcs.iter() {
		metadata.get_func_data(&func.name).clear_usage_info();
		let mut solver = Solver::new(func, metadata, &mut graph);
		solver.dfs(func.cfg.get_entry());
		metadata.get_func_data(&func.name).usage_info = solver.usage_info;
	}
	for global_var in program.global_vars.iter() {
		let mut queue = Vec::new();
		for func in program.funcs.iter() {
			if metadata.may_load(&func.name, &global_var.ident) {
				queue.push(func.name.clone());
			}
		}
		while let Some(u) = queue.pop() {
			for v in graph.get_neighbors(&u) {
				if metadata.may_load(&v, &global_var.ident) {
					continue;
				}
				metadata.get_func_data(&v).set_may_load(&global_var.ident);
			}
		}
		let mut queue = Vec::new();
		for func in program.funcs.iter() {
			if metadata.may_store(&func.name, &global_var.ident) {
				queue.push(func.name.clone());
			}
		}
		while let Some(u) = queue.pop() {
			for v in graph.get_neighbors(&u) {
				if metadata.may_store(&v, &global_var.ident) {
					continue;
				}
				metadata.get_func_data(&v).set_may_store(&global_var.ident);
			}
		}
	}
}
