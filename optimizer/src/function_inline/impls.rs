use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{function_inline::entry::FuncEntry, RrvmOptimizer};

use llvm::{
	JumpInstr, LlvmInstrTrait, LlvmInstrVariant::*, LlvmTemp, LlvmTempManager,
	PhiInstr, Value,
};
use rrvm::{
	basicblock::LlvmBasicBlock,
	program::{LlvmFunc, LlvmProgram},
	LlvmNode,
};
use utils::{errors::Result, math::increment, to_label, UseTemp};

use super::{entry::get_func_table, func_list::get_func_list, InlineFunction};

impl RrvmOptimizer for InlineFunction {
	fn new() -> Self {
		Self {}
	}
	fn apply(self, program: &mut LlvmProgram) -> Result<bool> {
		let func_list = get_func_list(program);
		if func_list.is_empty() {
			return Ok(false);
		}
		// eprintln!("befor inline\n{}\n", program);
		let table = get_func_table(func_list, program);
		program
			.funcs
			.iter_mut()
			.for_each(|func| inline(func, &table, &mut program.temp_mgr));
		// eprintln!("after inline\n{}\n", program);
		Ok(true)
	}
}

fn inline(
	func: &mut LlvmFunc,
	table: &HashMap<String, FuncEntry>,
	mgr: &mut LlvmTempManager,
) {
	let flag = func.cfg.blocks.iter().any(|v| {
		v.borrow()
			.instrs
			.iter()
			.any(|v| v.is_call() && table.contains_key(&v.get_label().name))
	});
	if flag {
		// let blocks = std::mem::take(&mut );
		let mut blocks = Vec::new();
		let mut edges = Vec::new();
		let mut id2node = HashMap::<i32, LlvmNode>::new();
		let mut phi_label_mapper_for_last = HashMap::new();
		for block in func.cfg.blocks.iter() {
			let block = &block.borrow();
			let w = block.weight;
			let mut last = LlvmBasicBlock::new(block.id, w);
			last.phi_instrs = block.phi_instrs.clone();
			for instr in block.instrs.iter() {
				match instr.get_variant() {
					CallInstr(instr) if table.contains_key(&instr.func.name) => {
						let entry = table.get(&instr.get_label().name).unwrap();
						let mut new_nodes = Vec::new();
						let mut id_mapper = HashMap::new(); // 用于重建边
						let mut temp_mapper: HashMap<LlvmTemp, Value> = HashMap::new(); // 用于重写读入变量
						let mut label_mapper = HashMap::new(); // 用于重写跳转指令
																			 // 计算上面几个 mapper
						for node in entry.nodes.iter() {
							let mut new_node = LlvmBasicBlock::new(
								increment(&mut func.total),
								w * node.weight,
							);
							id_mapper.insert(node.id, new_node.id);
							label_mapper.insert(node.label(), new_node.label());
							for instr in node.phi_instrs.iter() {
								let mut new_instr = instr.clone();
								if let Some(target) = instr.get_write() {
									let new_temp = mgr.new_temp(target.var_type, false);
									new_instr.set_target(new_temp.clone());
									temp_mapper.insert(target, new_temp.into());
								}
								new_node.phi_instrs.push(new_instr);
							}
							for instr in node.instrs.iter() {
								let mut new_instr = instr.clone_box();
								if let Some(target) = instr.get_write() {
									let new_temp = mgr.new_temp(target.var_type, false);
									new_instr.set_target(new_temp.clone());
									temp_mapper.insert(target, new_temp.into());
								}
								new_node.instrs.push(new_instr);
							}
							new_node.jump_instr =
								node.jump_instr.as_ref().map(|v| v.clone_box());
							new_node.kill_size = node.kill_size;
							new_nodes.push(new_node);
						}
						for (formal, (_, actual)) in
							entry.params.iter().zip(instr.params.iter())
						{
							temp_mapper.insert(formal.clone(), actual.clone());
						}
						for (u, v) in entry.edges.iter() {
							edges
								.push((*id_mapper.get(u).unwrap(), *id_mapper.get(v).unwrap()));
						}
						let func_entry_id =
							*id_mapper.get(&entry.nodes.first().unwrap().id).unwrap();
						edges.push((last.id, func_entry_id));
						last.jump_instr = Some(Box::new(JumpInstr {
							target: to_label(func_entry_id),
						}));
						let (last_id, node) = wrap(last);
						id2node.insert(last_id, node.clone());
						blocks.push(node);
						last = LlvmBasicBlock::new(increment(&mut func.total), w);
						let mut source = Vec::new();
						for mut node in new_nodes {
							node.map_temp(&temp_mapper);
							node.map_phi_label(&label_mapper);
							node.map_jump_label(&label_mapper);
							let instr = node.jump_instr.as_ref().unwrap();
							if let RetInstr(instr) = instr.get_variant() {
								if let Some(value) = instr.value.as_ref() {
									source.push((value.clone(), node.label()));
								}
								node.jump_instr = Some(Box::new(JumpInstr {
									target: last.label(),
								}));
								edges.push((node.id, last.id));
							}
							let (id, node) = wrap(node);
							id2node.insert(id, node.clone());
							blocks.push(node);
						}
						if !entry.var_type.is_void() {
							last.phi_instrs.push(PhiInstr {
								target: instr.target.clone(),
								var_type: instr.var_type,
								source,
							})
						}
					}
					_ => last.instrs.push(instr.clone_box()),
				}
			}
			for v in block.succ.iter() {
				edges.push((last.id, v.borrow().id))
			}
			last.kill_size = block.kill_size;
			last.jump_instr = block.jump_instr.as_ref().map(|v| v.clone_box());
			phi_label_mapper_for_last.insert(block.label(), last.label());
			let (id, node) = wrap(last);
			id2node.insert(id, node.clone());
			blocks.push(node);
		}
		// eprintln!("\n{:?}\n", phi_label_mapper_for_last);
		for block in blocks.iter() {
			block.borrow_mut().clear();
			block.borrow_mut().map_phi_label(&phi_label_mapper_for_last);
		}
		for (u, v) in edges {
			let x = id2node.get(&u).unwrap();
			let y = id2node.get(&v).unwrap();
			x.borrow_mut().succ.push(y.clone());
			y.borrow_mut().prev.push(x.clone());
		}
		for block in func.cfg.blocks.iter() {
			block.borrow_mut().clear();
		}
		func.cfg.blocks = blocks;
		func.cfg.analysis();
	}
}

fn wrap(block: LlvmBasicBlock) -> (i32, LlvmNode) {
	(block.id, Rc::new(RefCell::new(block)))
}
