use std::{
	cell::RefCell,
	collections::{HashMap, HashSet},
	rc::Rc,
};

use instr_schedule::instr_schedule_by_dag;
use instrdag::InstrDag;
use instruction::{riscv::prelude::*, temp::TempManager};
use rrvm::prelude::*;
use transformer::{to_riscv, to_rt_type};
use utils::{errors::Result, BLOCKSIZE_THRESHOLD, DEPENDENCY_EXPLORE_DEPTH};

pub mod instr_schedule;
pub mod instrdag;
pub mod remove_phi;
pub mod transformer;

pub fn get_functions(
	program: &mut RiscvProgram,
	funcs: Vec<LlvmFunc>,
) -> Result<()> {
	for func in funcs {
		let converted_func = convert_func(func, &mut program.temp_mgr)?;
		program.funcs.push(instr_schedule(
			converted_func.0,
			converted_func.1,
			converted_func.2,
			&mut program.temp_mgr,
		)?);
	}
	Ok(())
}

pub fn instr_schedule(
	func: RiscvFunc,
	live_ins: Vec<HashSet<RiscvTemp>>,
	live_outs: Vec<HashSet<RiscvTemp>>,
	mgr: &mut TempManager,
) -> Result<RiscvFunc> {
	func.cfg.clear_data_flow();
	func.cfg.analysis();
	let mut new_blocks = Vec::new();
	for (idx, node) in func.cfg.blocks.iter().enumerate() {
		let nodes =
			instr_schedule_block(node, &live_ins[idx], &live_outs[idx], mgr)?;
		new_blocks.extend(nodes);
	}
	Ok(RiscvFunc {
		total: mgr.total,
		spills: 0,
		cfg: RiscvCFG { blocks: new_blocks },
		name: func.name,
		params: func.params,
		ret_type: func.ret_type,
		external_resorce: HashSet::new(),
		entrance: Entrance::Unkonwn
	})
}
pub fn instr_schedule_block(
	riscv_node: &RiscvNode,
	live_ins: &HashSet<RiscvTemp>,
	live_outs: &HashSet<RiscvTemp>,
	mgr: &mut TempManager,
) -> Result<Vec<RiscvNode>> {
	let prev = riscv_node
		.borrow()
		.prev
		.iter()
		.map(|v| v.borrow().id)
		.collect::<HashSet<_>>();
	let succ = riscv_node
		.borrow()
		.succ
		.iter()
		.map(|v| v.borrow().id)
		.collect::<HashSet<_>>();
	// 判断 prev 和 succ 是否有交集
	if prev.intersection(&succ).count() > 0
		&& riscv_node.borrow().instrs.len() <= BLOCKSIZE_THRESHOLD
	{
		// filter call (instrs 中不能有 call 指令)
		if riscv_node.borrow().instrs.iter().any(|instr| instr.is_call()) {
			transform_basic_block_by_pipelining(riscv_node, live_ins, live_outs, mgr)
				.map(|v| vec![v])
		} else {
			transform_loop_block(riscv_node, mgr, 4)
		}
	} else {
		transform_basic_block_by_pipelining(riscv_node, live_ins, live_outs, mgr)
			.map(|v| vec![v])
	}
}
pub fn convert_func(
	func: LlvmFunc,
	mgr: &mut TempManager,
) -> Result<(RiscvFunc, Vec<HashSet<RiscvTemp>>, Vec<HashSet<RiscvTemp>>)> {
	let mut nodes = Vec::new();
	let mut edge = Vec::new();
	let mut table = HashMap::new();
	let mut alloc_table = HashMap::new();
	let mut live_ins = Vec::new();
	let mut live_outs = Vec::new();
	func.cfg.blocks.iter().for_each(remove_phi::remove_phi);
	for block in func.cfg.blocks.iter() {
		for instr in block.borrow().instrs.iter() {
			if let Some((temp, length)) = instr.get_alloc() {
				alloc_table.insert(temp, length);
			}
		}
	}
	for block in func.cfg.blocks.iter() {
		let live_in: HashSet<_> =
			block.borrow().live_in.iter().map(|v| mgr.get(v)).collect();
		let live_out: HashSet<_> =
			block.borrow().live_out.iter().map(|v| mgr.get(v)).collect();
		live_ins.push(live_in);
		live_outs.push(live_out);
	}
	for block in func.cfg.blocks {
		let kill_size = (block.borrow().kill_size + 15) & -16;
		let id = block.borrow().id;
		edge.extend(block.borrow().succ.iter().map(|v| (id, v.borrow().id)));
		let node = transform_basicblock(&block, mgr)?;
		table.insert(id, node.clone());
		if kill_size != 0 {
			let instr = if is_lower(kill_size) {
				ITriInstr::new(Addi, SP.into(), SP.into(), kill_size.into())
			} else {
				let num = load_imm(kill_size, &mut node.borrow_mut().instrs, mgr);
				RTriInstr::new(Add, SP.into(), SP.into(), num)
			};
			node.borrow_mut().instrs.push(instr);
		}
		let mut instrs =
			to_riscv(block.borrow().jump_instr.as_ref().unwrap(), mgr)?;
		node.borrow_mut().set_jump(instrs.pop());
		node.borrow_mut().instrs.extend(instrs);
		nodes.push(node);
	}
	for (u, v) in edge {
		force_link_node(table.get(&u).unwrap(), table.get(&v).unwrap())
	}
	Ok((
		RiscvFunc {
			total: mgr.total,
			spills: 0,
			cfg: RiscvCFG { blocks: nodes },
			name: func.name,
			params: func.params,
			ret_type: func.ret_type,
			entrance: Entrance::Unkonwn,
			external_resorce: HashSet::new(),
		},
		live_ins,
		live_outs,
	))
}

fn transform_loop_block(
	node: &RiscvNode,
	_mgr: &mut TempManager,
	_n: usize, // 展开次数
) -> Result<Vec<RiscvNode>> {
	// calc T_0
	let r = [1, 1, 1, 1, 2]; // mem,br,mul/div,floating-point,sum
												 //按照RT 求出总的资源占用，再和 R 中各项相除求得最大值
	let mut rt = [0, 0, 0, 0, 0];
	for instr in node.borrow().instrs.iter() {
		let rt_vec = to_rt_type(instr);
		for i in 0..5 {
			rt[i] += rt_vec[i];
		}
	}
	let mut t0 = 0;
	for i in 0..5 {
		t0 = t0.max((rt[i] + r[i] - 1) / r[i]);
	}
	// 模数变量扩展
	// 找到本循环内 def 且 use 非 live_in 非 live_out 的变量
	let mut tmps = HashSet::new();
	for tmp in node.borrow().defs.intersection(&node.borrow().uses) {
		if !node.borrow().live_in.contains(tmp)
			&& !node.borrow().live_out.contains(tmp)
		{
			tmps.insert(*tmp);
		}
	}
	// 建立数据依赖图
	let mut dag = HashMap::new();
	// 先加上非数组的边
	for (idx, instr) in node.borrow().instrs.iter().enumerate() {
		let read_tmps = instr.get_riscv_read();
		for &i in read_tmps.clone().iter() {
			let mut _alpha = -1;
			for j in (0..idx).rev() {
				let optime = node.borrow().instrs[j].get_rtn_array()[4];
				if node.borrow().instrs[j].get_riscv_write().contains(&i) {
					_alpha = j as i32;
					// 往 dag 里面加边
					dag
						.entry((j as i32, idx))
						.and_modify(|e: &mut Vec<(i32, i32)>| e.push((0, optime)))
						.or_insert(vec![(0, optime)]);
					break;
				}
				if _alpha != -1 {
					// 按照该指令的后一条周期往前遍历
					for k in idx..node.borrow().instrs.len() {
						let optime = node.borrow().instrs[k].get_rtn_array()[4];
						if node.borrow().instrs[k].get_riscv_write().contains(&i) {
							dag
								.entry((k as i32, idx))
								.and_modify(|e: &mut Vec<(i32, i32)>| e.push((1, optime)))
								.or_insert(vec![(1, optime)]);
							break;
						}
					}
				}
			}
		}
	}
	// 再加上数组的边,从当前到 DEPENDENCY_EXPLORE_DEPTH
	// 对于数组中的某个元素，判断它在一个周期内的增量是否是常数
	let mut taint_map: HashMap<(i32, RiscvTemp), Vec<(i32, RiscvTemp)>> =
		HashMap::new();
	let mut store_map: HashMap<RiscvImm, usize> = HashMap::new();
	// 判断 load 和 store 的 dependency
	// 先找到 store 的元素
	for (idx, instr) in node.borrow().instrs.iter().enumerate() {
		if instr.is_store().unwrap_or(false) {
			store_map.insert(instr.get_imm().unwrap(), idx);
			if let Some(OffsetReg(offset, base)) = instr.get_imm() {
				let mut regs = HashSet::new();
				regs.insert(instr.get_riscv_read()[0]);
				let mut relevant_imms = Vec::new();
				// reverse taint analysis
				for i in (0..idx).rev() {
					let write_regs = node.borrow().instrs[i].get_riscv_write();
					// judge if write_regs and taint_map[(offset,base)] has intersection
					if HashSet::from_iter(write_regs.iter().cloned())
						.intersection(&regs)
						.count() > 0
					{
						if node.borrow().instrs[i].is_load().unwrap_or(false) {
							if let OffsetReg(offset, base) =
								node.borrow().instrs[i].get_imm().unwrap()
							{
								relevant_imms.push((offset, base));
							} else {
								unreachable!();
							}
						} else {
							regs.extend(node.borrow().instrs[i].get_riscv_read());
						}
					}
				}
				taint_map.entry((offset, base)).or_default().extend(relevant_imms);
			} else {
				unreachable!();
			}
		}
	}
	// taint_map filter entries that are not empty
	taint_map.retain(|_, v| !v.is_empty());
	// 对于 taint_map keys 和 values 里面的 register，找到每个周期的增量，只考虑加减常数，mov 指令
	let mut taint_regs: HashSet<RiscvTemp> =
		taint_map.keys().map(|&(_offset, base)| base).collect();
	let mut reg_increments: HashMap<RiscvTemp, IncrementType> = HashMap::new();
	for (_key, val) in taint_map.iter() {
		for (_idx, reg) in val.iter() {
			taint_regs.insert(*reg);
		}
	}
	let mut map_increments = HashMap::new();
	let mut first_write_idx = None;
	for reg in taint_regs.iter() {
		// iterate to first write
		for (idx, instr) in node.borrow().instrs.iter().enumerate() {
			if instr.get_riscv_write().contains(reg) {
				match instr.get_increment() {
					IncrementType::Int(i) => {
						map_increments
							.entry(reg)
							.and_modify(|v: &mut Vec<(RiscvTemp, IncrementType, usize)>| {
								v.push((instr.get_riscv_read()[0], IncrementType::Int(i), idx))
							})
							.or_insert(vec![(
								instr.get_riscv_read()[0],
								IncrementType::Int(i),
								idx,
							)]);
						first_write_idx = Some(idx);
						break;
					}
					IncrementType::LongLong(i) => {
						map_increments
							.entry(reg)
							.and_modify(|v: &mut Vec<(RiscvTemp, IncrementType, usize)>| {
								v.push((
									instr.get_riscv_read()[0],
									IncrementType::LongLong(i),
									idx,
								))
							})
							.or_insert(vec![(
								instr.get_riscv_read()[0],
								IncrementType::LongLong(i),
								idx,
							)]);
						first_write_idx = Some(idx);
						break;
					}
					_ => {
						break;
					}
				}
			}
		}
		if let Some(first_write) = first_write_idx {
			for i in
				(first_write + 1..node.borrow().instrs.len()).chain(0..=first_write + 1)
			{
				let instr = &node.borrow().instrs[i];
				let regs = map_increments.get(reg).cloned().unwrap();
				// judge if (more than) one of instr's write reg is contained in map_increments' values
				let mut entry_read_update = None;
				let mut entry_write_update = None;
				for (reg, _increments, _idx) in regs.iter() {
					if instr.get_riscv_read().contains(reg) {
						match instr.get_increment() {
							IncrementType::Int(i1) => {
								entry_read_update = Some((
									instr.get_riscv_write()[0],
									reg,
									IncrementType::Int(i1),
									i,
								));
								break;
							}
							IncrementType::LongLong(i1) => {
								entry_read_update = Some((
									instr.get_riscv_write()[0],
									reg,
									IncrementType::LongLong(i1),
									i,
								));
								break;
							}
							_ => {
								break;
							}
						}
					}
					if instr.get_riscv_write().contains(reg) {
						match instr.get_increment() {
							IncrementType::Int(i1) => {
								entry_write_update = Some((
									instr.get_riscv_read()[0],
									reg,
									IncrementType::Int(i1),
									i,
								));
							}
							IncrementType::LongLong(i1) => {
								entry_write_update = Some((
									instr.get_riscv_read()[0],
									reg,
									IncrementType::LongLong(i1),
									i,
								));
							}
							_ => {
								break;
							}
						}
					}
				}
				// update map_increments
				// todo 想一下如果中间 map_increments 中含有的寄存器被写了怎么办
				if let Some((write_reg, read_reg, offset, i)) =
					entry_read_update.clone()
				{
					// 在 map_increments[reg] 中找到含有 read_reg 的项并且记录下offset， 检查是否含有 write_reg 的项，如果没有就插入，否则更新write_reg 的项
					let mut map_offset = IncrementType::None;
					for (reg, increments, _) in map_increments.get(reg).unwrap().iter() {
						if reg == read_reg {
							map_offset = increments.clone();
							break;
						}
					}
					if let IncrementType::None = map_offset {
						unreachable!();
					} else if let Some(vec) = map_increments.get_mut(reg) {
						let mut is_update = false;
						for (reg, offset_old, _) in vec.iter_mut() {
							if *reg == write_reg {
								*offset_old = map_offset.clone() - offset.clone();
								is_update = true;
								break;
							}
						}
						if !is_update {
							vec.push((write_reg, map_offset - offset, i));
						}
					} else {
						unreachable!();
					}
					if let Some((write_reg, _read_reg, _offset, _i)) = entry_write_update
					{
						if entry_read_update.is_none() {
							// 从 map_increments[reg] 中删除 含有 write_reg的项
							if let Some(vec) = map_increments.get_mut(reg) {
								vec.retain(|(reg, _, _)| *reg != write_reg);
							} else {
								unreachable!();
							}
						}
					}
				}
			}
		}
	}
	// fill reg_increments 为每个周期增减常数的寄存器
	for reg in taint_regs.iter() {
		let increments = map_increments.get(reg).cloned().unwrap();
		for (reg_1, increment, _) in increments.iter() {
			if *reg_1 == *reg {
				reg_increments.insert(*reg, IncrementType::None - increment.clone());
			}
		}
	}
	// 对于 taint_map 的每一项，看常数增量的步频是否相等，再做一次filter
	let mut taint_map_filtered = HashMap::new();
	for (offset, store_reg) in taint_map.keys() {
		if reg_increments.keys().any(|v| v == store_reg) {
			let increments = taint_map.get(&(*offset, *store_reg)).unwrap();
			let mut filtered_increments = Vec::new();
			for (increment, read_reg) in increments.iter() {
				if reg_increments.keys().any(|v| v == read_reg) {
					// find out if read_reg is in map_increments[store_reg] 's values's firsts
					if map_increments
						.get(store_reg)
						.cloned()
						.unwrap()
						.iter()
						.any(|(reg, _increment, _)| reg == read_reg)
						&& reg_increments[store_reg] == reg_increments[read_reg]
					{
						filtered_increments.push((increment, read_reg));
					}
				}
			}
			if !filtered_increments.is_empty() {
				taint_map_filtered.insert((*offset, *store_reg), filtered_increments);
			}
		}
	}
	// 对taint_map_filtered 中的每一项，从0..DEPENDENCY_EXPLORE_DEPTH 看有没有可以加上边的地方
	for (offset, store_reg) in taint_map_filtered.keys() {
		let store_incre = reg_increments[store_reg].clone();
		// 找 store 和 read 之间的依赖关系，算第0周期store下去的值在第几周期可以被读出来，第0周期偏移量就是 offset
		let loads =
			taint_map_filtered.get(&(*offset, *store_reg)).cloned().unwrap();
		for (offset_read, read_reg) in loads.iter() {
			let t1 = &map_increments
				.get(store_reg)
				.unwrap()
				.iter()
				.find(|(reg, _increment, _instr)| reg == *read_reg)
				.unwrap()
				.1;
			let t = t1.clone();
			let read_instr_cnt = map_increments
				.get(store_reg)
				.cloned()
				.unwrap()
				.iter()
				.find(|(reg, _increment, _instr)| reg == *read_reg)
				.unwrap()
				.2;
			match t {
				IncrementType::Int(i) => {
					let read_incre = &reg_increments[*read_reg];
					let dist = store_incre.clone() - IncrementType::Int(i);
					let mut init_dist = IncrementType::Int(**offset_read) + dist;
					for i in 0..DEPENDENCY_EXPLORE_DEPTH {
						if IncrementType::Int(*offset) == init_dist {
							dag
								.entry((
									store_map[&OffsetReg(*offset, *store_reg)] as i32,
									read_instr_cnt,
								))
								.and_modify(|e: &mut Vec<(i32, i32)>| e.push((i, 1)))
								.or_insert(vec![(i, 1)]);
						}
						init_dist = init_dist + read_incre.clone();
					}
				}
				IncrementType::LongLong(i) => {
					let read_incre = &reg_increments[*read_reg];
					let dist = store_incre.clone() - IncrementType::LongLong(i);
					let mut init_dist = IncrementType::Int(**offset_read) + dist;
					for i in 0..DEPENDENCY_EXPLORE_DEPTH {
						if IncrementType::Int(*offset) == init_dist {
							dag
								.entry((
									store_map[&OffsetReg(*offset, *store_reg)] as i32,
									read_instr_cnt,
								))
								.and_modify(|e: &mut Vec<(i32, i32)>| e.push((i, 1)))
								.or_insert(vec![(i, 1)]);
						}
						init_dist = init_dist + read_incre.clone();
					}
				}
				_ => {}
			}
		}
	}
	// now the dag should be set
	// get T_0 range by max(\sum_{loop}(alpha)/\sum_{loop}(d))
	// iterate the loops in dag
	let _alpha_sum = 0;
	let _d_sum = 0;
	// Iterate over the nodes in the DAG
	for (node, _) in dag.iter() {
		let mut visited = HashSet::new();
		let mut stack = vec![*node];

		// Perform depth-first search
		while let Some(current) = stack.pop() {
			if visited.contains(&current) {
				// Cycle detected, current node is part of a loop
				// Handle the loop as needed
				// ...
			} else {
				visited.insert(current);

				// Add the neighbors of the current node to the stack
				if let Some(neighbors) = dag.get(&current) {
					for _neighbor in neighbors {
						todo!();
					}
				}
			}
		}
	}
	Err(utils::SysycError::RiscvGenError(
		"Loop block not supported".to_string(),
	))
}
fn transform_basic_block_by_pipelining(
	node: &RiscvNode,
	live_in: &HashSet<RiscvTemp>,
	live_out: &HashSet<RiscvTemp>,
	_mgr: &mut TempManager,
) -> Result<RiscvNode> {
	let instr_dag = InstrDag::new(node)?;
	for i in node.borrow().instrs.iter() {
		println!("{}", i);
	}
	println!("-----------------");
	let liveliness_map = get_liveliness_map(node, live_in, live_out);
	let mut block = BasicBlock::new(node.borrow().id, node.borrow().weight);
	block.kill_size = node.borrow().kill_size;
	block.instrs = instr_schedule_by_dag(instr_dag, liveliness_map)?;
	for i in block.instrs.iter() {
		println!("{}", i);
	}
	Ok(Rc::new(RefCell::new(block)))
}
#[derive(Clone, Debug)]
pub struct Liveliness {
	is_livein: bool,
	is_liveout: bool,
	use_num: usize,
}
fn get_liveliness_map(
	node: &RiscvNode,
	live_in: &HashSet<RiscvTemp>,
	live_out: &HashSet<RiscvTemp>,
) -> HashMap<RiscvTemp, Liveliness> {
	let mut map = HashMap::new();
	for instr in node.borrow().instrs.iter() {
		let read_tmps = instr.get_riscv_read();
		let write_tmps = instr.get_riscv_write();
		for tmp in read_tmps.iter() {
			map
				.entry(*tmp)
				.or_insert(Liveliness {
					is_livein: false,
					is_liveout: false,
					use_num: 0,
				})
				.use_num += 1;
		}
		for tmp in write_tmps.iter() {
			map
				.entry(*tmp)
				.or_insert(Liveliness {
					is_livein: false,
					is_liveout: false,
					use_num: 0,
				})
				.is_liveout = true;
		}
	}
	// do live_in
	for tmp in live_in.iter() {
		map
			.entry(*tmp)
			.or_insert(Liveliness {
				is_livein: true,
				is_liveout: false,
				use_num: 0,
			})
			.is_livein = true;
	}
	for tmp in live_out.iter() {
		map
			.entry(*tmp)
			.or_insert(Liveliness {
				is_livein: false,
				is_liveout: true,
				use_num: 0,
			})
			.is_liveout = true;
	}
	map
}
fn transform_basicblock(
	node: &LlvmNode,
	mgr: &mut TempManager,
) -> Result<RiscvNode> {
	// 先识别该基本块是否是基本本块（循环内只有一个基本块的情况），判断其前驱后继是否含有同一个基本块
	let instrs: Result<Vec<_>, _> =
		node.borrow().instrs.iter().map(|v| to_riscv(v, mgr)).collect();
	let mut block = BasicBlock::new(node.borrow().id, node.borrow().weight);
	block.kill_size = node.borrow().kill_size;
	block.instrs = instrs?.into_iter().flatten().collect();
	let riscv_node = Rc::new(RefCell::new(block));
	Ok(riscv_node)
}
