use super::{
	utils::{
		calc_mod, get_entry, get_typed_one, get_typed_zero, is_constant_term,
		topology_sort, ModStatus,
	},
	CalcCoef,
};
use crate::{
	calc_coef::utils::{
		calc_arith, calc_call, calc_ret, create_wrapper, get_constant_term, Entry,
	},
	metadata::FuncData,
	MetaData, RrvmOptimizer,
};
use core::panic;
use llvm::{
	LlvmInstrTrait,
	LlvmInstrVariant::{
		AllocInstr, ArithInstr, CallInstr, CompInstr, ConvertInstr, GEPInstr,
		JumpCondInstr, LoadInstr, PhiInstr, StoreInstr,
	},
	LlvmTemp, LlvmTempManager, Value,
};
use rrvm::{
	cfg::{BasicBlock, CFG},
	program::{LlvmFunc, LlvmProgram},
};
use std::{
	cell::RefCell,
	collections::{HashMap, HashSet, VecDeque},
	mem,
	rc::Rc,
	vec,
};
use utils::{errors::Result, Label};
impl RrvmOptimizer for CalcCoef {
	fn new() -> Self {
		Self {}
	}
	fn apply(
		self,
		program: &mut LlvmProgram,
		metadata: &mut MetaData,
	) -> Result<bool> {
		let old_len = program.funcs.len();
		let new_funcs: Vec<_> = mem::take(&mut program.funcs)
			.into_iter()
			.flat_map(|func| {
				let ord_blocks = topology_sort(&func);
				if ord_blocks.is_empty() {
					return vec![func];
				}
				if can_calc(&func, metadata) {
					for index in func.params.iter().map(|x| x.unwrap_temp().unwrap()) {
						let calc_funcs = calc_coef(
							&func,
							index,
							&mut program.temp_mgr,
							ord_blocks.clone(),
							metadata.get_func_data(&func.name),
						);
						if calc_funcs.len() == 2 {
							return calc_funcs;
						}
					}
					vec![func]
				} else {
					vec![func]
				}
			})
			.collect();
		program.funcs = new_funcs;
		Ok(old_len != program.funcs.len())
	}
}
fn can_calc(func: &LlvmFunc, metadata: &mut MetaData) -> bool {
	// 按照以下条件进行判断：1. 参数>=2 2. 有大于等于两次递归 3. 递归函数中有相同项（用 gvn 判断）4. 没有 load/store 没有 convert 没有 alloc gep
	if func.params.len() < 2 {
		return false;
	}
	for block in func.cfg.blocks.iter() {
		for instr in block.borrow().instrs.iter() {
			match instr.get_variant() {
				LoadInstr(_) | StoreInstr(_) | AllocInstr(_) | ConvertInstr(_)
				| GEPInstr(_) => {
					return false;
				}
				_ => {}
			}
		}
	}
	let mut call_selfs = vec![];
	for i in func.cfg.blocks.iter() {
		for instr in i.borrow().instrs.iter() {
			if let CallInstr(callinstr) = instr.get_variant() {
				if callinstr.func.name == func.name {
					call_selfs.push(callinstr.clone());
				}
			}
		}
	}
	if call_selfs.len() < 2 {
		return false;
	}
	let func_data = metadata.get_func_data(&func.name);
	let mut params_data = Vec::new();
	for call_instr in call_selfs.iter() {
		let nums = call_instr
			.params
			.iter()
			.filter(|(_, val)| matches!(val, Value::Temp(_)))
			.map(|(_, val)| {
				func_data.get_number(&val.unwrap_temp().unwrap()).unwrap()
			})
			.collect::<Vec<_>>();
		if params_data.is_empty() {
			params_data = nums.to_vec();
		} else if !nums.iter().zip(params_data.iter()).any(|(a, b)| a == b) {
			return false;
		}
	}
	if params_data.is_empty() {
		return false;
	}
	true
}
#[allow(clippy::borrowed_box, clippy::too_many_arguments)]
fn map_instr(
	instr: &Box<dyn LlvmInstrTrait>,
	entry_map: &mut HashMap<LlvmTemp, Entry>,
	block_instrs: &mut Vec<Box<dyn LlvmInstrTrait>>,
	mgr: &mut LlvmTempManager,
	block_phi_instrs: &mut Vec<llvm::PhiInstr>,
	addr: &LlvmTemp,
	func_name: String,
	params_len: usize,
	index: usize,
) -> bool {
	match instr.get_variant() {
		ArithInstr(arith_instr) => {
			return calc_arith(arith_instr, entry_map, block_instrs, mgr, params_len);
		}
		CompInstr(comp_instr) => {
			// 要求 lhs rhs 的 data 系数为 0
			let lhs = comp_instr.lhs.clone();
			let rhs = comp_instr.rhs.clone();
			let target = comp_instr.target.clone();
			let get_lhs_val = get_constant_term(&lhs, entry_map);
			let get_rhs_val = get_constant_term(&rhs, entry_map);
			if let Some(lhs_val) = get_lhs_val {
				if let Some(rhs_val) = get_rhs_val {
					let my_target = mgr.new_temp(llvm::VarType::I32, false);
					let instr = llvm::CompInstr {
						target: my_target.clone(),
						lhs: lhs_val.clone(),
						rhs: rhs_val,
						op: comp_instr.op,
						var_type: lhs_val.get_type(),
						kind: comp_instr.kind,
					};
					block_instrs.push(Box::new(instr));
					entry_map.insert(
						target,
						Entry {
							k_val: vec![Value::Int(0); params_len],
							b_val: Value::Temp(my_target),
							mod_val: ModStatus::new(),
							params_len,
						},
					);
				} else {
					return false;
				}
			} else {
				return false;
			}
		}
		JumpCondInstr(jump_cond_instr) => {
			// 同上要求 cond 和 data 无关
			let cond = jump_cond_instr.cond.clone();
			let get_cond_val = get_constant_term(&cond, entry_map);
			if let Some(cond_val) = get_cond_val {
				let instr = llvm::JumpCondInstr {
					cond: cond_val.clone(),
					target_true: jump_cond_instr.target_true.clone(),
					target_false: jump_cond_instr.target_false.clone(),
					var_type: cond_val.get_type(),
				};
				block_instrs.push(Box::new(instr));
			} else {
				return false;
			}
		}
		PhiInstr(phi_instr) => {
			// 想一下怎么处理有 phi 的情况
			// 处理有 phi 的情况，搞成多个 phi
			let target = phi_instr.target.clone();
			let new_sources_k: Vec<_> = phi_instr
				.source
				.iter()
				.map(|(val, label)| {
					let get_val = {
						if let Value::Temp(t) = val {
							let entry = entry_map.get(t);
							if let Some(entry) = entry {
								entry.k_val.clone()
							} else {
								panic!("phi instr val not in entry map");
							}
						} else {
							vec![Value::Int(0); params_len]
						}
					};
					(get_val, label.clone())
				})
				.collect();
			let new_sources_b: Vec<_> = phi_instr
				.source
				.iter()
				.map(|(val, label)| {
					let get_val = {
						if let Value::Temp(t) = val {
							let entry = entry_map.get(t);
							if let Some(entry) = entry {
								entry.b_val.clone()
							} else {
								panic!("phi instr val not in entry map");
							}
						} else {
							val.clone()
						}
					};
					(get_val, label.clone())
				})
				.collect();
			let mut k_targets = vec![];
			let entries: Vec<_> = phi_instr
				.source
				.iter()
				.map(|(val, _label)| get_entry(val, entry_map, params_len))
				.collect();
			if entries.iter().any(|x| x.is_none()) {
				return false;
			}
			let entries_unwrapped: Vec<_> =
				entries.iter().map(|x| x.clone().unwrap()).collect();
			let instr: Box<dyn LlvmInstrTrait> = Box::new(phi_instr.clone());
			let status = calc_mod(&instr, entries_unwrapped);
			if status.is_none() {
				return false;
			}
			for i in 0..params_len {
				let k_target = mgr.new_temp(phi_instr.var_type, false);
				let instr = llvm::PhiInstr {
					target: k_target.clone(),
					source: new_sources_k
						.clone()
						.into_iter()
						.map(|(val, label)| (val[i].clone(), label))
						.collect::<Vec<_>>(),
					var_type: phi_instr.var_type,
				};
				block_instrs.push(Box::new(instr.clone()));
				block_phi_instrs.push(instr);
				k_targets.push(k_target);
			}
			let b_target = mgr.new_temp(phi_instr.var_type, false);
			let instr2 = llvm::PhiInstr {
				target: b_target.clone(),
				source: new_sources_b.clone(),
				var_type: phi_instr.var_type,
			};
			block_instrs.push(Box::new(instr2.clone()));
			block_phi_instrs.push(instr2);
			entry_map.insert(
				target,
				Entry {
					k_val: k_targets.into_iter().map(Value::Temp).collect(),
					b_val: Value::Temp(b_target),
					mod_val: status.unwrap(),
					params_len,
				},
			);
		}
		CallInstr(call_instr) => {
			// 检查是否是 call 的自身，如果不是的话，params 中都不能与 data 有关
			return calc_call(
				call_instr,
				entry_map,
				block_instrs,
				mgr,
				func_name,
				addr,
				index,
			);
		}
		llvm::LlvmInstrVariant::RetInstr(retinstr) => {
			return calc_ret(retinstr, entry_map, block_instrs, mgr, addr);
		}
		llvm::LlvmInstrVariant::JumpInstr(instr) => {
			block_instrs.push(Box::new(instr.clone()));
		}
		_ => {
			unreachable!("instr not supported");
		}
	}
	true
}
pub fn judge_return(entries: &[Entry]) -> Option<Option<Value>> {
	let mut imms = vec![];
	let mut mod_val = None;
	for entry in entries.iter() {
		// 判断是否是立即数
		if is_constant_term(entry) {
			if let Value::Int(i) = entry.b_val {
				imms.push(i);
				continue;
			}
		}
		// 判断模数
		let mod_num = entry.mod_val.mod_val.clone();
		if let Some(mod_num) = mod_num {
			if entry.mod_val.is_activated {
				if mod_val.is_none() {
					mod_val = Some(mod_num);
				} else if mod_val != Some(mod_num) {
					return None;
				}
			} else {
				return None;
			}
		} else if mod_val.is_some() {
			return None;
		}
	}
	for imm in imms.iter() {
		// 判断所有 imm 都小于除数的绝对值
		if let Some(Value::Int(mod_val)) = mod_val.clone() {
			if imm.abs() >= mod_val.abs() {
				return None;
			}
		}
	}
	Some(mod_val)
}
type Blocks = Vec<Rc<RefCell<BasicBlock<Box<dyn LlvmInstrTrait>, LlvmTemp>>>>;
#[allow(clippy::too_many_arguments)]
fn map_coef_instrs(
	func: &LlvmFunc,
	index: LlvmTemp,
	addr: LlvmTemp,
	mgr: &mut LlvmTempManager,
	special_nodes: HashSet<i32>,
	recurse_index: Vec<Box<dyn LlvmInstrTrait>>, // 生成 recursive_index 的 instrs
	my_index: LlvmTemp,                          // recursive index
	block_ord: Vec<i32>,
	my_recurse_index: LlvmTemp,
) -> Option<(Blocks, Option<Value>)> {
	let params_len = func.params.len() - 1;
	let mut entry_map = HashMap::new();
	let index_pos =
		func.params.iter().position(|x| *x == Value::Temp(index.clone())).unwrap();
	let data: Vec<llvm::Value> = func
		.params
		.iter()
		.filter(|x| **x != Value::Temp(index.clone()))
		.cloned()
		.collect();
	entry_map.insert(
		index.clone(),
		Entry {
			k_val: vec![Value::Int(0); params_len],
			b_val: Value::Temp(my_index.clone()),
			mod_val: ModStatus::new(),
			params_len,
		},
	);
	for (idx, i) in data.iter().enumerate() {
		if let Value::Temp(tmp) = i {
			// 只有第i项为 Int(1) 其他项为 Int(0)的 vector
			let k_val = (0..params_len)
				.map(|i| {
					if i == idx {
						get_typed_one(tmp)
					} else {
						get_typed_zero(tmp)
					}
				})
				.collect();
			entry_map.insert(
				tmp.clone(),
				Entry {
					k_val,
					b_val: get_typed_zero(tmp),
					mod_val: ModStatus::new(),
					params_len,
				},
			);
		}
	}
	let mut new_instrs = vec![];
	let mut res_vec = vec![];
	for recurse in recurse_index.iter() {
		let res = map_instr(
			recurse,
			&mut entry_map,
			&mut res_vec,
			mgr,
			&mut vec![],
			&addr,
			func.name.clone(),
			func.params.len() - 1,
			index_pos,
		);
		if !res {
			return None;
		}
	}
	let call_instr = llvm::CallInstr {
		target: mgr.new_temp(llvm::VarType::I32, false),
		var_type: llvm::VarType::Void,
		func: Label::new(format!("{}_calc_coef", func.name)),
		params: vec![
			(llvm::VarType::I32Ptr, Value::Temp(addr.clone())),
			(
				llvm::VarType::I32,
				entry_map.get(&my_recurse_index).unwrap().b_val.clone(),
			),
		],
	};
	let mut phi_instrs = vec![];
	let mut jmp_instrs = vec![];
	// 先把 data 和 index 放进entry_map 因为自有 Value 所以不用搞 instrs
	for id in block_ord.iter() {
		let block = func.cfg.blocks.iter().find(|x| x.borrow().id == *id).unwrap();
		let mut block_instrs: Vec<Box<dyn LlvmInstrTrait>> = vec![];
		let mut block_phi_instrs: Vec<llvm::PhiInstr> = vec![];
		if special_nodes.contains(&block.borrow().id) {
			for i in res_vec.iter() {
				block_instrs.push(i.clone());
			}
			block_instrs.push(Box::new(call_instr.clone()));
		}
		let has_jmp = block.borrow().jump_instr.is_some();
		let jmp_vec = {
			if let Some(instr) = block.borrow().jump_instr.clone() {
				vec![instr]
			} else {
				vec![]
			}
		};
		for instr in block.borrow().instrs.iter().chain(jmp_vec.iter()) {
			let res = map_instr(
				instr,
				&mut entry_map,
				&mut block_instrs,
				mgr,
				&mut block_phi_instrs,
				&addr,
				func.name.clone(),
				params_len,
				index_pos,
			);
			if !res {
				return None;
			}
		}
		if has_jmp {
			let jmp_instr = block_instrs.pop().unwrap();
			jmp_instrs.push(Some(jmp_instr));
		} else {
			jmp_instrs.push(None);
		}
		phi_instrs.push(block_phi_instrs);
		new_instrs.push(block_instrs);
	}
	// assemble entries with return values' maps
	let mut ret_entries_vec = vec![];
	let mut ret_imms = vec![]; // 得到返回值之后再判断
	for block in func.cfg.blocks.iter() {
		if let Some(jmp_instr) = block.borrow().jump_instr.clone() {
			if let llvm::LlvmInstrVariant::RetInstr(retinstr) =
				jmp_instr.get_variant()
			{
				let val = retinstr.value.clone();
				if let Some(val) = val {
					match val {
						Value::Temp(t) => {
							let entry = entry_map.get(&t);
							if let Some(entry) = entry {
								ret_entries_vec.push(entry.clone());
							} else {
								panic!("ret instr val not in entry map");
							}
						}
						Value::Int(i) => {
							ret_imms.push(i);
						}
						_ => {}
					}
				}
			}
		}
	}
	let mod_val = judge_return(&ret_entries_vec);
	mod_val.as_ref()?;
	let unwrapped_mod_val = mod_val.unwrap();
	if let Some(mod_val) = &unwrapped_mod_val {
		for imm in ret_imms.iter() {
			if let Value::Int(i) = mod_val {
				if imm.abs() >= (*i).abs() {
					return None;
				}
			}
		}
	}
	// assemble blocks with phi_instrs and new_instrs
	let mut new_blocks = vec![];
	for block in func
		.cfg
		.blocks
		.iter()
		.zip(new_instrs.iter())
		.zip(phi_instrs.iter())
		.zip(jmp_instrs.iter())
	{
		let (((block, instrs), phi_instrs), jmp_instr) = block;
		let new_block = block.clone();
		new_block.borrow_mut().instrs.clone_from(instrs);
		new_block.borrow_mut().phi_instrs.clone_from(phi_instrs);
		new_block.borrow_mut().jump_instr.clone_from(jmp_instr);
		new_blocks.push(new_block);
	}
	Some((new_blocks, unwrapped_mod_val))
}
fn calc_coef(
	func: &LlvmFunc,
	index: LlvmTemp,
	mgr: &mut LlvmTempManager,
	block_ord: Vec<i32>,
	funcdata: &mut FuncData,
) -> Vec<LlvmFunc> {
	let data = func
		.params
		.iter()
		.filter(|x| **x != Value::Temp(index.clone()))
		.cloned()
		.collect::<Vec<_>>();
	//  多源 bfs
	// 找到所有特殊点，即是有递归调用自身的点
	let mut special_map = HashMap::new();
	for block in func.cfg.blocks.iter() {
		for instr in block.borrow().instrs.iter() {
			if let CallInstr(callinstr) = instr.get_variant() {
				if callinstr.func.name == func.name {
					special_map.insert(block.borrow().id, block.clone());
				}
			}
		}
	}
	// bfs 算特可达点
	loop {
		// calculate special reachables
		let mut special_reachable_map = HashMap::new();
		let mut queue = VecDeque::new();
		for node in special_map.keys() {
			queue.push_back(special_map.get(node).unwrap().clone());
		}
		while let Some(node) = queue.pop_front() {
			if special_reachable_map.contains_key(&node.borrow().id) {
				continue;
			}
			special_reachable_map.insert(node.borrow().id, node.clone());
			for succ in node.borrow().succ.iter() {
				queue.push_back(succ.clone());
			}
		}
		// calculate special nodes
		let mut new_special_map = HashMap::new();
		for node in special_map.keys() {
			if !special_map
				.get(node)
				.unwrap()
				.borrow()
				.prev
				.iter()
				.any(|v| special_reachable_map.contains_key(&v.borrow().id))
			{
				new_special_map.insert(*node, special_map.get(node).unwrap().clone());
			} else if !special_map
				.get(node)
				.unwrap()
				.borrow()
				.prev
				.iter()
				.all(|v| special_reachable_map.contains_key(&v.borrow().id))
			{
				let borrowed_node = special_map.get(node).unwrap().borrow();
				let filtered_prevs = borrowed_node
					.prev
					.iter()
					.filter(|v| !special_reachable_map.contains_key(&v.borrow().id));
				new_special_map
					.extend(filtered_prevs.map(|v| (v.borrow().id, v.clone())));
			}
		}
		let is_changed = special_map.keys().collect::<HashSet<_>>()
			!= new_special_map.keys().collect::<HashSet<_>>();
		if !is_changed {
			break;
		} else {
			special_map = new_special_map;
		}
	}

	let addr = mgr.new_temp(llvm::VarType::I32Ptr, false);
	let my_index = mgr.new_temp(index.var_type, false);
	// calculate recurse index
	let index_pos =
		func.params.iter().position(|x| *x == Value::Temp(index.clone())).unwrap();
	// 检查，找所有 call 自身指令
	let copied_func = LlvmFunc {
		total: func.total,
		spills: func.spills,
		cfg: CFG {
			blocks: func.cfg.blocks.clone(),
		},
		name: func.name.clone(),
		ret_type: func.ret_type,
		params: func.params.clone(),
	};

	let mut recurse_num = None;
	for i in func.cfg.blocks.iter() {
		for instr in i.borrow().instrs.iter() {
			if let CallInstr(callinstr) = instr.get_variant() {
				if callinstr.func.name == func.name {
					let (_ty, recurse_tmp) = callinstr.params[index_pos].clone();
					if let Value::Temp(t) = recurse_tmp {
						if recurse_num.is_none() {
							recurse_num = Some(t);
						} else {
							// t 和 recurse_num 不能相等
							if funcdata.get_number(&t).unwrap()
								!= funcdata.get_number(&recurse_num.clone().unwrap()).unwrap()
							{
								return vec![copied_func];
							}
						}
					}
				}
			}
		}
	}
	if recurse_num.is_none() {
		return vec![copied_func];
	}
	let recurse_tmp = recurse_num.unwrap();
	let mut recurse_idx = vec![];
	let mut reads = VecDeque::new();
	reads.push_back(recurse_tmp.clone());
	while !reads.is_empty() {
		let tmp = reads.pop_front().unwrap();
		// 找到所有使用 tmp 的指令
		for i in func.cfg.blocks.iter() {
			for instr in i.borrow().instrs.iter() {
				if let Some(write) = instr.get_write() {
					if write == tmp {
						recurse_idx.push(instr.clone());
						for i in instr.get_read() {
							reads.push_back(i);
						}
						break;
					}
				}
			}
		}
	}
	recurse_idx.reverse();
	let new_blocks = map_coef_instrs(
		func,
		index.clone(),
		addr.clone(),
		mgr,
		special_map.keys().cloned().collect(),
		recurse_idx,
		my_index.clone(),
		block_ord,
		recurse_tmp,
	);
	if new_blocks.is_none() {
		return vec![copied_func];
	}
	let (new_blocks, mod_val) = new_blocks.unwrap();
	let calc_func = LlvmFunc {
		total: mgr.total as i32,
		spills: 0,
		cfg: rrvm::cfg::CFG { blocks: new_blocks },
		name: format!("{}_calc_coef", func.name),
		ret_type: llvm::VarType::Void,
		params: vec![Value::Temp(addr.clone()), Value::Temp(my_index)],
	};
	let node = create_wrapper(
		&data.iter().map(|x| x.unwrap_temp().unwrap()).collect(),
		&index,
		func.name.clone(),
		mgr,
		func.params.len() - 1,
		mod_val,
	);
	let wrapper_func = LlvmFunc {
		total: mgr.total as i32,
		spills: 0,
		cfg: CFG { blocks: vec![node] },
		name: func.name.clone(),
		ret_type: func.ret_type,
		params: func.params.clone(),
	};
	vec![wrapper_func, calc_func]
}