use std::collections::{HashMap, HashSet, VecDeque};

use super::RemoveUselessCode;
use crate::{metadata::MetaData, RrvmOptimizer};
use llvm::{LlvmInstr, LlvmInstrVariant::*, LlvmTemp};
use rrvm::{dominator::*, program::LlvmProgram, LlvmCFG};
use utils::{errors::Result, Label, UseTemp};

#[derive(Hash, PartialEq, Eq, Clone)]
enum Node {
	Instr(LlvmTemp),
	Block(Label),
}

impl From<LlvmTemp> for Node {
	fn from(temp: LlvmTemp) -> Self {
		Self::Instr(temp)
	}
}

impl From<Label> for Node {
	fn from(label: Label) -> Self {
		Self::Block(label)
	}
}

impl RrvmOptimizer for RemoveUselessCode {
	fn new() -> Self {
		Self {}
	}
	fn apply(
		self,
		program: &mut LlvmProgram,
		metadata: &mut MetaData,
	) -> Result<bool> {
		fn solve(cfg: &mut LlvmCFG, metadata: &mut MetaData) -> bool {
			let mut dom_tree = LlvmDomTree::new(cfg, true);
			let mut flag = false;
			let mut graph = HashMap::new();
			let mut visited = HashSet::new();
			let mut queue = VecDeque::new();

			let mut add_edge = |u: Node, v: Node| {
				graph.entry(u).or_insert_with(HashSet::new).insert(v);
			};

			let mut insert_worklist = |node: Node| {
				if !visited.contains(&node) {
					visited.insert(node.clone());
					queue.push_back(node);
				}
			};

			let mut has_sideeffect = |instr: &LlvmInstr| match instr.get_variant() {
				CallInstr(instr) => !metadata.get_func_data(&instr.func.name).pure,
				_ => instr.has_sideeffect(),
			};

			for block in cfg.blocks.iter() {
				let block = block.borrow();
				for instr in block.instrs.iter() {
					if has_sideeffect(instr) {
						for v in instr.get_read() {
							insert_worklist(v.into());
						}
						insert_worklist(block.label().into());
					} else {
						let u = instr.get_write().unwrap();
						for v in instr.get_read() {
							add_edge(u.clone().into(), v.into());
						}
						add_edge(u.clone().into(), block.label().into());
					}
				}
				for instr in block.phi_instrs.iter() {
					let u = instr.get_write().unwrap();
					add_edge(u.clone().into(), block.label().into());
					for v in instr.get_read() {
						add_edge(u.clone().into(), v.into());
					}
					for (_, label) in instr.source.iter() {
						add_edge(u.clone().into(), label.clone().into());
					}
				}
				if let Some(instr) = block.jump_instr.as_ref() {
					if instr.is_ret() {
						insert_worklist(block.label().into());
						for v in instr.get_read() {
							insert_worklist(v.into());
						}
					} else {
						for v in instr.get_read() {
							add_edge(block.label().into(), v.into());
						}
					}
				}
				for v in dom_tree.get_df(block.id) {
					add_edge(block.label().into(), v.borrow().label().into());
				}
				if block.prev.len() > 1 {
					for v in block.prev.iter() {
						add_edge(block.label().into(), v.borrow().label().into());
					}
				}
			}

			while let Some(node) = queue.pop_front() {
				for v in graph.remove(&node).unwrap_or_default() {
					if !visited.contains(&v) {
						visited.insert(v.clone());
						queue.push_back(v);
					}
				}
			}

			for block in cfg.blocks.iter_mut() {
				let mut block = block.borrow_mut();
				block.instrs.retain(|instr| {
					has_sideeffect(instr)
						|| instr.get_write().map_or(true, |v| visited.contains(&v.into()))
						|| {
							flag = true;
							false
						}
				});
				block.phi_instrs.retain(|instr| {
					instr.get_write().map_or(true, |v| visited.contains(&v.into())) || {
						flag = true;
						false
					}
				});
			}

			let mut mapper = HashMap::new();

			for block in cfg.blocks.iter_mut() {
				let block = block.borrow();
				if !visited.contains(&block.label().into()) {
					let mut dom = dom_tree.get_dominator(block.id).unwrap();
					let dom = loop {
						if visited.contains(&dom.borrow().label().into()) {
							break dom;
						}
						let dom_id = dom.borrow().id;
						if let Some(new_dom) = dom_tree.get_dominator(dom_id) {
							dom = new_dom;
						} else {
							break dom;
						}
					};
					flag = true;
					mapper.insert(block.label(), dom.clone());
				}
			}

			let label_mapper =
				mapper.iter().map(|(k, v)| (k.clone(), v.borrow().label())).collect();

			for block in cfg.blocks.iter() {
				let mut block = block.borrow_mut();
				block.jump_instr.as_mut().unwrap().map_label(&label_mapper);
				let succ = std::mem::take(&mut block.succ);
				block.succ = succ
					.into_iter()
					.map(|v| {
						let label = v.borrow().label();
						mapper.get(&label).cloned().unwrap_or(v)
					})
					.collect();
			}

			cfg.resolve_prev();
			flag
		}

		Ok(
			program
				.funcs
				.iter_mut()
				.fold(false, |last, func| solve(&mut func.cfg, metadata) || last),
		)
	}
}
