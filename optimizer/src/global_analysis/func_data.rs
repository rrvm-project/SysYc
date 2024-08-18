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

use super::{impls::BUILTIN_FUNCS, var_data};

#[derive(Default)]
struct CallGraph {
	edges: HashMap<String, HashSet<String>>,
}

impl CallGraph {
	pub fn add_edge(&mut self, from: &str, to: &str) {
		self.edges.entry(from.to_owned()).or_default().insert(to.to_owned());
	}
	pub fn get_neighbors(&self, func: &str) -> HashSet<String> {
		self.edges.get(func).cloned().unwrap_or_default()
	}
}

struct Solver<'a> {
	func_name: String,
	dom_tree: LlvmDomTree,
	temp_mapper: HashMap<LlvmTemp, VarIdent>,
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
		let mut temp_mapper = HashMap::new();
		for (index, param) in func.params.iter().enumerate() {
			if param.get_type().is_ptr() {
				let addr = param.unwrap_temp().unwrap();
				temp_mapper.insert(addr, (func.name.clone(), index));
			}
		}
		Self {
			func_name: func.name.clone(),
			dom_tree: LlvmDomTree::new(&func.cfg, false),
			usage_info: UsageInfo::default(),
			temp_mapper,
			metadata,
			graph,
		}
	}
	pub fn dfs(&mut self, node: LlvmNode) {
		let block = &mut node.borrow_mut();
		for instr in block.phi_instrs.iter() {
			if instr.var_type.is_ptr() {
				let src_addr = instr.source.iter().find_map(|(value, _)| {
					value.unwrap_temp().and_then(|v| self.temp_mapper.get(&v))
				});
				if let Some(ident) = src_addr {
					self.temp_mapper.insert(instr.target.clone(), ident.clone());
				}
			}
		}
		block.instrs.retain(|instr| {
			match instr.get_variant() {
				LoadInstr(instr) => {
					if let Value::Temp(addr) = &instr.addr {
						if addr.is_global {
							self
								.temp_mapper
								.insert(instr.target.clone(), (addr.name.clone(), 0));
						} else if let Some(global_var) = self.temp_mapper.get(addr) {
							self.usage_info.may_loads.insert(global_var.clone());
						}
					}
				}
				GEPInstr(instr) => {
					if let Value::Temp(addr) = &instr.addr {
						if let Some(global_var) = self.temp_mapper.get(addr) {
							self.temp_mapper.insert(instr.target.clone(), global_var.clone());
						}
					}
				}
				StoreInstr(instr) => {
					if let Value::Temp(addr) = &instr.addr {
						if let Some(global_var) = self.temp_mapper.get(addr) {
							self.usage_info.may_stores.insert(global_var.clone());
							let var_data = self.metadata.get_var_data(global_var);
							if !var_data.to_load {
								return false;
							}
						}
					}
				}
				CallInstr(instr) => {
					self.graph.add_edge(&instr.func.name, &self.func_name);
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

	let source_names = program
		.global_vars
		.iter()
		.map(|v| v.ident.as_str())
		.chain(["系统调用"].iter().copied());
	let func_names = program
		.funcs
		.iter()
		.map(|f| f.name.as_str())
		.chain(BUILTIN_FUNCS.iter().copied())
		.map(|f| f.to_owned());

	for ident in source_names {
		let mut queue = Vec::new();
		for name in func_names.clone() {
			if metadata.may_load(&name, ident) {
				queue.push(name);
			}
		}
		while let Some(u) = queue.pop() {
			for v in graph.get_neighbors(&u) {
				if metadata.may_load(&v, ident) {
					continue;
				}
				metadata.get_func_data(&v).set_may_load((ident.to_owned(), 0));
				queue.push(v);
			}
		}
		let mut queue = Vec::new();
		for name in func_names.clone() {
			if metadata.may_store(&name, ident) {
				queue.push(name);
			}
		}
		while let Some(u) = queue.pop() {
			for v in graph.get_neighbors(&u) {
				if metadata.may_store(&v, ident) {
					continue;
				}
				metadata.get_func_data(&v).set_may_store((ident.to_owned(), 0));
				queue.push(v);
			}
		}
	}
}
