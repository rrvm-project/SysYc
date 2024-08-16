use std::collections::{HashMap, HashSet};

use super::{
	utils::{
		Addr, AddrInfo, ArrayInfo, ArrayState, MemItem, UseState, UseStateItem,
	},
	Mem2Reg,
};
use crate::{
	metadata::MetaData,
	number::{str2num, Number},
	RrvmOptimizer,
};
use llvm::{LlvmInstrVariant::*, LlvmTemp, LlvmTempManager, Value, VarType};
use rand::{rngs::StdRng, SeedableRng};
use rrvm::{
	dominator::LlvmDomTree,
	program::{LlvmFunc, LlvmProgram},
	LlvmNode,
};
use utils::{errors::Result, Label, MEM_TO_REG_LIMIT};

struct Solver<'a> {
	func_name: String,
	dom_tree: LlvmDomTree,
	rng: StdRng,
	metadata: &'a mut MetaData,
	mgr: &'a mut LlvmTempManager,
	addrs: HashSet<Addr>,
	base_addrs: HashMap<Number, Vec<Addr>>,
	addr_mapper: HashMap<LlvmTemp, Addr>,
	addr_info: HashMap<Addr, AddrInfo>,
	phi: HashMap<i32, HashSet<Addr>>,
	addr2temp: HashMap<i32, HashMap<Addr, Value>>,
	instance_phi: HashMap<(i32, Addr), LlvmTemp>,
	array_states: HashMap<i32, ArrayState>,
	use_states: HashMap<i32, UseStateItem>,
	global_base: HashSet<Number>,
	stack: Vec<(LlvmNode, HashSet<Number>)>,
}

impl<'a> Solver<'a> {
	pub fn new(
		func: &LlvmFunc,
		mgr: &'a mut LlvmTempManager,
		metadata: &'a mut MetaData,
	) -> Self {
		let mut addr_mapper = HashMap::new();
		let mut global_base = HashSet::new();
		let func_data = metadata.get_func_data(&func.name);
		for param in func.params.iter() {
			if param.get_type().is_ptr() {
				let temp = param.unwrap_temp().unwrap();
				let number = func_data.get_number(&temp).unwrap();
				addr_mapper.insert(temp, Addr::new(number.clone(), 0u32.into()));
				global_base.insert(number.clone());
			}
		}
		for store_var in func_data.usage_info.may_stores.iter() {
			let base = str2num(store_var);
			global_base.insert(base);
		}
		Self {
			func_name: func.name.clone(),
			dom_tree: LlvmDomTree::new(&func.cfg, false),
			rng: StdRng::from_entropy(),
			base_addrs: HashMap::new(),
			addr_info: HashMap::new(),
			addrs: HashSet::new(),
			array_states: HashMap::new(),
			phi: HashMap::new(),
			addr2temp: HashMap::new(),
			instance_phi: HashMap::new(),
			use_states: HashMap::new(),
			stack: Vec::new(),
			mgr,
			global_base,
			addr_mapper,
			metadata,
		}
	}
	fn get_addr(&self, temp: &LlvmTemp) -> Addr {
		self.addr_mapper.get(temp).unwrap().clone()
	}
	fn try_get_addr(&self, temp: &LlvmTemp) -> Option<Addr> {
		self.addr_mapper.get(temp).cloned()
	}
	fn set_number(&mut self, temp: LlvmTemp) {
		self
			.metadata
			.get_func_data(&self.func_name)
			.set_number(temp, Number::new(&mut self.rng))
	}
	fn get_number(&mut self, temp: &LlvmTemp) -> Option<Number> {
		self.metadata.get_func_data(&self.func_name).get_number(temp).cloned()
	}
	fn get_val_number(&mut self, value: &Value) -> Option<Number> {
		self.metadata.get_func_data(&self.func_name).get_val_number(value)
	}
	fn get_val_addr(&self, value: &Value) -> Addr {
		value.unwrap_temp().map(|temp| self.get_addr(&temp)).unwrap()
	}
	fn addr_anticipate(&self, id: i32, addr: &Addr) -> bool {
		self.addr2temp.get(&id).map(|v| v.contains_key(addr)).unwrap_or(false)
	}

	// part1: get all address that used in function
	pub fn calc_addr(
		&mut self,
		node: LlvmNode,
		mut addr2temp: HashMap<Addr, Value>,
	) {
		for instr in node.borrow().phi_instrs.iter() {
			if instr.var_type.is_ptr() {
				let src_addr = instr.source.iter().find_map(|(value, _)| {
					self.try_get_addr(&value.unwrap_temp().unwrap())
				});
				let number = self.get_number(&instr.target).unwrap();
				let addr = Addr::new(src_addr.unwrap().base, number);
				self.addr_mapper.insert(instr.target.clone(), addr);
			}
		}
		for instr in node.borrow().instrs.iter() {
			match instr.get_variant() {
				AllocInstr(instr) => {
					let number = self.get_number(&instr.target).unwrap();
					let addr = Addr::new(number.clone(), 0u32.into());
					self.addr_mapper.insert(instr.target.clone(), addr);
				}
				LoadInstr(instr) => {
					let temp = instr.addr.unwrap_temp().unwrap();
					if temp.is_global {
						let number = self.get_number(&temp).unwrap();
						let addr = Addr::new(number.clone(), 0u32.into());
						addr2temp.insert(addr.clone(), instr.target.clone().into());
						self.addr_mapper.insert(instr.target.clone(), addr);
						self.global_base.insert(number.clone());
					} else {
						self.addrs.insert(self.get_addr(&temp));
					}
				}
				StoreInstr(instr) => {
					let temp = instr.addr.unwrap_temp().unwrap();
					self.addrs.insert(self.get_addr(&temp));
				}
				CallInstr(instr) => {
					for (var_type, param) in instr.params.iter() {
						if var_type.is_ptr() {
							let temp = param.unwrap_temp().unwrap();
							self.addrs.insert(self.get_addr(&temp));
						}
					}
				}
				GEPInstr(instr) => {
					let offset = self.get_val_number(&instr.offset).unwrap();
					let base =
						self.addr_mapper.get(&instr.addr.unwrap_temp().unwrap()).unwrap();
					let addr = Addr::new(base.base.clone(), &base.offset + offset);
					addr2temp.insert(addr.clone(), instr.target.clone().into());
					self.addr_mapper.insert(instr.target.clone(), addr);
				}
				_ => {}
			}
		}
		for v in self.dom_tree.get_children(node.borrow().id).clone() {
			self.calc_addr(v, addr2temp.clone());
		}
		self.addr2temp.insert(node.borrow().id, addr2temp);
	}

	// part2: get all places that value is defined
	fn insert_def(&mut self, addr: Addr, block_id: i32) {
		self.addr_info.entry(addr).or_default().insert_def(block_id);
	}
	pub fn calc_defs(&mut self, func: &LlvmFunc) {
		for addr in self.addrs.iter() {
			self.base_addrs.entry(addr.base.clone()).or_default().push(addr.clone());
		}
		for node in func.cfg.blocks.iter() {
			let block = node.borrow();
			let mut info = ArrayInfo::default();
			for addr in self.addrs.iter() {
				info.insert(addr.clone());
			}
			for instr in block.instrs.iter() {
				match instr.get_variant() {
					StoreInstr(instr) => {
						let addr = self.get_val_addr(&instr.addr);
						let addrs = info.solve_conflict(&addr);
						info.insert(addr.clone()); // 维护活跃位置
						self.insert_def(addr, block.id);
						for addr in addrs {
							self.insert_def(addr, block.id);
						}
					}

					CallInstr(instr) => {
						for (index, (var_type, param)) in instr.params.iter().enumerate() {
							if var_type.is_ptr()
								&& self
									.metadata
									.get_var_data(&(instr.func.name.clone(), index))
									.to_store
							{
								let addr = self.get_val_addr(param);
								for addr in info.remove(&addr.base) {
									self.insert_def(addr, block.id);
								}
							}
						}
						let func_data = self.metadata.get_func_data(&instr.func.name);
						for global_var in func_data.usage_info.may_stores.iter() {
							let base = str2num(global_var);
							let addrs = info.remove(&base);
							for addr in addrs {
								self.addr_info.entry(addr).or_default().insert_def(block.id);
							}
						}
					}
					_ => {}
				}
			}
		}
	}

	// part3: get all places that phi instr is needed
	pub fn calc_phi(&mut self) {
		for (addr, addr_info) in self.addr_info.iter_mut() {
			let defs = &mut addr_info.defs;
			let mut queue = defs.iter().copied().collect::<Vec<_>>();
			while let Some(u) = queue.pop() {
				for v in self.dom_tree.get_df(u) {
					let vid = v.borrow().id;
					if !defs.contains(&vid) {
						defs.insert(vid);
						queue.push(vid);
					}
				}
			}
			for block_id in defs.iter() {
				self.phi.entry(*block_id).or_default().insert(addr.clone());
			}
		}
	}

	// part4: solve load instruction
	fn get_value(
		&mut self,
		addr: Addr,
		item: &MemItem,
		var_type: VarType,
	) -> Value {
		match item {
			MemItem::Value(value) => value.clone(),
			MemItem::PhiDef(block_id) => self
				.instance_phi
				.entry((*block_id, addr.clone()))
				.or_insert_with(|| self.mgr.new_temp(var_type, false))
				.clone()
				.into(),
		}
	}
	fn map_load_instr(&mut self, node: &LlvmNode, array_state: &mut ArrayState) {
		let prev_ids: Vec<_> =
			node.borrow().prev.iter().map(|v| v.borrow().id).collect();
		let mut block = node.borrow_mut();
		let instrs = std::mem::take(&mut block.instrs);
		let mut phi_info = ArrayInfo::default();
		for addr in self.phi.get(&block.id).unwrap_or(&HashSet::new()).iter() {
			array_state.remove(addr);
			if prev_ids.iter().all(|id| self.addr_anticipate(*id, addr))
				&& !prev_ids.is_empty()
			{
				array_state.insert_item(addr.clone(), block.id);
			}
			phi_info.insert(addr.clone());
		}
		block.instrs = instrs
			.into_iter()
			.flat_map(|mut ori_instr| {
				ori_instr.map_temp(&array_state.temp_mapper);
				match ori_instr.get_variant() {
					LoadInstr(instr) => {
						if instr.addr.is_global() {
							vec![ori_instr]
						} else {
							let addr = self.get_val_addr(&instr.addr);
							if let Some(item) = array_state.get(&addr) {
								array_state.insert(
									instr.target.clone(),
									self.get_value(addr, item, instr.var_type),
								);
								vec![]
							} else {
								array_state.load(addr, instr.target.clone().into());
								vec![ori_instr]
							}
						}
					}
					StoreInstr(instr) => {
						let addr = self.get_val_addr(&instr.addr);
						phi_info.solve_conflict(&addr);
						array_state.store(addr, instr.value.clone());
						vec![ori_instr]
					}
					CallInstr(instr) => {
						for (index, (var_type, param)) in instr.params.iter().enumerate() {
							if var_type.is_ptr()
								&& self
									.metadata
									.get_var_data(&(instr.func.name.clone(), index))
									.to_store
							{
								let addr = self.get_val_addr(param);
								array_state.remove_base(&addr.base);
							}
						}
						let func_data = self.metadata.get_func_data(&instr.func.name);
						for global_var in func_data.usage_info.may_stores.iter() {
							let base = str2num(global_var);
							array_state.remove_base(&base);
							phi_info.remove(&base);
						}
						vec![ori_instr]
					}
					_ => vec![ori_instr],
				}
			})
			.collect();
		block.jump_instr.as_mut().unwrap().map_temp(&array_state.temp_mapper);
		let node_label = block.label();
		fn map_value(
			instrs: &mut [llvm::PhiInstr],
			array_state: &ArrayState,
			label: &Label,
		) {
			for instr in instrs.iter_mut() {
				for (value, instr_label) in instr.source.iter_mut() {
					if label == instr_label {
						*value = array_state.map_value(value);
					}
				}
			}
		}
		for v in block.succ.clone() {
			if std::ptr::eq(v.as_ptr(), node.as_ptr()) {
				map_value(&mut block.phi_instrs, array_state, &node_label)
			} else {
				map_value(&mut v.borrow_mut().phi_instrs, array_state, &node_label)
			}
		}
	}
	fn calc_load(&mut self, node: LlvmNode, mut array_state: ArrayState) {
		self.map_load_instr(&node, &mut array_state);
		let children = self.dom_tree.get_children(node.borrow().id).clone();
		for v in children {
			self.calc_load(v, array_state.clone());
		}
		self.array_states.insert(node.borrow().id, array_state);
	}
	pub fn solve_load_instr(&mut self, func: &LlvmFunc) {
		self.calc_load(func.cfg.get_entry(), ArrayState::default());
	}

	// part5: solve phi instruction
	pub fn solve_phi_instr(&mut self, func: &LlvmFunc) {
		let mut phi_temps: Vec<_> = self.instance_phi.clone().into_iter().collect();
		let id2block: HashMap<_, _> =
			func.cfg.blocks.iter().map(|v| (v.borrow().id, v.clone())).collect();
		while let Some(((id, addr), target)) = phi_temps.pop() {
			let node = id2block.get(&id).unwrap();
			let prev = node.borrow().prev.clone();
			// TODO: Partial Redundancy Elimination
			let source = prev
				.iter()
				.map(|v| {
					let mut v = v.borrow_mut();
					let item = self.array_states.get(&v.id).unwrap().get(&addr);
					let value = match item {
						Some(MemItem::Value(value)) => value.clone(),
						Some(MemItem::PhiDef(id)) => {
							if let Some(temp) = self.instance_phi.get(&(*id, addr.clone())) {
								temp.clone().into()
							} else {
								let temp = self.mgr.new_temp(target.var_type, false);
								self.instance_phi.insert((*id, addr.clone()), temp.clone());
								phi_temps.push(((*id, addr.clone()), temp.clone()));
								temp.into()
							}
						}
						None => {
							let temp = self.mgr.new_temp(target.var_type, false);
							let addr = self.addr2temp.get(&v.id).unwrap().get(&addr).unwrap();
							let instr = llvm::LoadInstr {
								addr: addr.clone(),
								target: temp.clone(),
								var_type: target.var_type,
							};
							self.set_number(temp.clone());
							v.instrs.push(Box::new(instr));
							temp.into()
						}
					};
					(value, v.label())
				})
				.collect();
			self.set_number(target.clone());
			let instr = llvm::PhiInstr::new(target, source);
			node.borrow_mut().phi_instrs.push(instr);
		}
	}

	// part6: solve store instruction
	pub fn calc_use_state(&mut self, func: &LlvmFunc) {
		let mut changed;
		for block in func.cfg.blocks.iter() {
			self.use_states.insert(block.borrow().id, UseStateItem::default());
		}
		let (_, loop_info) = func.cfg.loop_analysis();
		loop {
			changed = false;
			for block in func.cfg.blocks.iter() {
				let block = block.borrow();
				let range = loop_info.get(&block.id).unwrap().borrow().loop_range;
				let mut loads = HashSet::new();
				let mut stores = HashMap::new();
				let mut iter = block.succ.iter();
				if let Some(v) = iter.next() {
					let state = &self.use_states.get(&v.borrow().id).unwrap().state_in;
					loads.clone_from(&state.loads);
					stores.clone_from(&state.stores);
					for v in iter {
						let state = &self.use_states.get(&v.borrow().id).unwrap().state_in;
						stores.retain(|addr, range| {
							if let Some(v_range) = state.stores.get(addr) {
								range.extend(v_range);
								true
							} else {
								false
							}
						});
						loads.extend(state.loads.iter().cloned());
					}
				} else if func.name != "main" {
					loads = self
						.addrs
						.iter()
						.filter(|addr| self.global_base.contains(&addr.base))
						.cloned()
						.collect();
				}
				let state_out = UseState {
					loads: loads.clone(),
					stores: stores.clone(),
				};
				for instr in block.instrs.iter().rev() {
					match instr.get_variant() {
						LoadInstr(instr) => {
							if !instr.addr.unwrap_temp().unwrap().is_global {
								let addr = self.get_val_addr(&instr.addr);
								stores.retain(|v, _| v.base != addr.base);
								loads.insert(addr);
							}
						}
						StoreInstr(instr) => {
							let addr = self.get_val_addr(&instr.addr);
							if stores.get(&addr).map_or(true, |v| !range.contains(v))
								&& loads.iter().any(|v| addr.base == v.base)
							{
								if let Some(v) = stores.get_mut(&addr) {
									v.shirink(&range);
								} else {
									stores.insert(addr, range);
								}
							}
						}
						CallInstr(instr) => {
							for (index, (var_type, param)) in instr.params.iter().enumerate()
							{
								if var_type.is_ptr() {
									let addr = self.get_val_addr(param);
									if var_type.is_ptr() {
										let var_data = self
											.metadata
											.get_var_data(&(instr.func.name.clone(), index));
										if var_data.to_load {
											let addrs =
												self.base_addrs.get(&addr.base).unwrap().clone();
											loads.extend(addrs);
										}
									}
								}
							}
							let func_data = self.metadata.get_func_data(&instr.func.name);
							for global_var in func_data.usage_info.may_loads.iter() {
								let base = str2num(global_var);
								stores.retain(|v, _| v.base != base);
								let addrs =
									self.base_addrs.get(&base).cloned().unwrap_or_default();
								loads.extend(addrs);
							}
						}
						_ => {}
					}
				}
				let state_in = UseState { loads, stores };
				let new_state = UseStateItem {
					state_in,
					state_out,
				};
				if &new_state != self.use_states.get(&block.id).unwrap() {
					changed = true;
					self.use_states.insert(block.id, new_state);
				}
			}
			if !changed {
				break;
			}
		}
	}
	pub fn solve_store_instr(&mut self, func: &LlvmFunc) {
		self.calc_use_state(func);
		let (_, loop_info) = func.cfg.loop_analysis();
		for block in func.cfg.blocks.iter() {
			let block = &mut block.borrow_mut();
			let range = loop_info.get(&block.id).unwrap().borrow().loop_range;
			let mut state = self.use_states.remove(&block.id).unwrap().state_out;
			state.stores.clear();
			block.instrs.reverse();
			block.instrs.retain(|instr| match instr.get_variant() {
				LoadInstr(instr) => {
					if !instr.addr.unwrap_temp().unwrap().is_global {
						let addr = self.get_val_addr(&instr.addr);
						state.stores.retain(|v, _| v.base != addr.base);
						state.loads.insert(addr);
					}
					true
				}
				StoreInstr(instr) => {
					let addr = self.get_val_addr(&instr.addr);
					state.stores.get(&addr).map_or(true, |v| !range.contains(v))
						&& state.loads.iter().any(|v| addr.base == v.base)
						&& {
							if let Some(v) = state.stores.get_mut(&addr) {
								v.shirink(&range);
							} else {
								state.stores.insert(addr, range);
							}
							true
						}
				}
				CallInstr(instr) => {
					for (index, (var_type, param)) in instr.params.iter().enumerate() {
						if var_type.is_ptr() {
							let addr = self.get_val_addr(param);
							let var_data =
								self.metadata.get_var_data(&(instr.func.name.clone(), index));
							if var_data.to_load {
								let addrs = self.base_addrs.get(&addr.base).unwrap().clone();
								state.loads.extend(addrs);
							}
						}
					}

					let func_data = self.metadata.get_func_data(&instr.func.name);
					for global_var in func_data.usage_info.may_loads.iter() {
						let base = str2num(global_var);
						state.stores.retain(|v, _| v.base != base);
						let addrs = self.base_addrs.get(&base).cloned().unwrap_or_default();
						state.loads.extend(addrs);
					}
					true
				}
				_ => true,
			});
			block.instrs.reverse();
		}
	}

	// optional part 1: hoist load instruction
	fn load_hoisting(&mut self, node: &LlvmNode) -> HashSet<Number> {
		let mut block = node.borrow_mut();
		let init_weight = block.weight;
		let mut store_base = HashSet::new();
		for addr in self.phi.get(&block.id).unwrap_or(&HashSet::new()).iter() {
			store_base.insert(addr.base.clone());
		}
		block.instrs.retain(|instr| match instr.get_variant() {
			LoadInstr(instr) => {
				if instr.addr.unwrap_temp().unwrap().is_global {
					true
				} else {
					let addr = self.get_val_addr(&instr.addr);
					store_base.contains(&addr.base) || {
						let mut best_weight = init_weight * 0.99;
						let mut best_node = None;
						for (node, store_base) in self.stack.iter().rev() {
							if !self.addr_anticipate(node.borrow().id, &addr) {
								break;
							}
							let weight = node.borrow().weight;
							if weight < best_weight {
								best_weight = weight;
								best_node = Some(node);
							}
							if store_base.contains(&addr.base) {
								break;
							}
						}
						if let Some(best_node) = best_node {
							best_node.borrow_mut().instrs.push(Box::new(instr.clone()));
							false
						} else {
							true
						}
					}
				}
			}
			StoreInstr(instr) => {
				let addr = self.get_val_addr(&instr.addr);
				store_base.insert(addr.base);
				true
			}
			CallInstr(instr) => {
				for (index, (var_type, param)) in instr.params.iter().enumerate() {
					if var_type.is_ptr() {
						let addr = self.get_val_addr(param);
						let var_data =
							self.metadata.get_var_data(&(instr.func.name.clone(), index));
						if var_data.to_store {
							store_base.insert(addr.base);
						}
					}
				}
				let func_data = self.metadata.get_func_data(&instr.func.name);
				for global_var in func_data.usage_info.may_stores.iter() {
					let base = str2num(global_var);
					if self.global_base.contains(&base) {
						store_base.insert(base);
					}
				}
				true
			}
			_ => true,
		});
		store_base
	}
	fn solve_load_hoisting(&mut self, node: LlvmNode) {
		let children = self.dom_tree.get_children(node.borrow().id).clone();
		let store_base = self.load_hoisting(&node);
		self.stack.push((node.clone(), store_base));
		for v in children {
			self.solve_load_hoisting(v);
		}
		self.stack.pop();
	}
}

impl RrvmOptimizer for Mem2Reg {
	fn new() -> Self {
		Self {}
	}

	fn apply(
		self,
		program: &mut LlvmProgram,
		metadata: &mut MetaData,
	) -> Result<bool> {
		fn solve(
			func: &LlvmFunc,
			mgr: &mut LlvmTempManager,
			metadata: &mut MetaData,
		) -> bool {
			let mut solver = Solver::new(func, mgr, metadata);
			solver.calc_addr(func.cfg.get_entry(), HashMap::new());
			if solver.addrs.is_empty() || solver.addrs.len() > MEM_TO_REG_LIMIT {
				return false;
			}
			solver.calc_defs(func);
			solver.calc_phi();
			solver.solve_load_instr(func);
			solver.solve_phi_instr(func);
			solver.solve_store_instr(func);
			solver.solve_load_hoisting(func.cfg.get_entry());
			false
		}

		Ok(program.funcs.iter().fold(false, |last, func| {
			solve(func, &mut program.temp_mgr, metadata) || last
		}))
	}
}
