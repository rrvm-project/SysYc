use std::{cell::RefCell, collections::HashMap, rc::Rc};

use llvm::{
	CompOp, JumpInstr, LlvmInstr, LlvmInstrTrait, LlvmTemp, LlvmTempManager,
	Value,
};
use log::trace;
use rrvm::{
	cfg::unlink_node,
	program::LlvmFunc,
	rrvm_loop::{
		loop_info::{get_loop_info::get_loop_info, LoopType, SimpleLoopInfo},
		LoopPtr,
	},
	LlvmNode,
};

const UNROLL_CNT: usize = 4;

#[allow(unused)]
// 首先获得 loop 所包含的所有基本块，确保循环只有一个 exit 和一个 exit prev，
// 在 get_loop_info 中确保循环内指令总量较少，只有一个 into_entry, 只有一条回边
// 然后确定展开次数
pub fn loop_unroll(
	func: &mut LlvmFunc,
	loop_: LoopPtr,
	temp_mgr: &mut LlvmTempManager,
) {
	let cfg = &mut func.cfg;
	let func_params = &func.params;
	if !loop_.borrow().no_inner {
		return;
	}
	let mut loop_bbs: Vec<LlvmNode> = Vec::new();
	let mut stack = Vec::new();
	let mut insert_loop_bbs = |bb: LlvmNode| {
		if !loop_bbs.contains(&bb) {
			loop_bbs.push(bb);
		}
	};
	stack.push(loop_.borrow().header.clone());
	while let Some(stack_bb) = stack.pop() {
		if stack_bb.borrow().loop_.as_ref().is_some_and(|l| *l == loop_) {
			insert_loop_bbs(stack_bb.clone());
			stack.append(&mut stack_bb.borrow().dominates_directly.clone());
		}
	}
	// 确保循环只有一个 exit 和一个 exit_prev
	let mut exit_bb = None;
	let mut exit_prev = None;
	let mut check = true;
	for bb in loop_bbs.iter() {
		if !check {
			break;
		}
		for succ in bb.borrow().succ.iter() {
			if !succ.borrow().loop_.as_ref().is_some_and(|l| *l == loop_) {
				if exit_bb.as_ref().is_some() {
					check = false;
					break;
				}
				exit_bb = Some(succ.clone());
				exit_prev = Some(bb.clone());
			}
		}
	}
	if exit_bb.is_none() || !check {
		return;
	}
	let loop_info = get_loop_info(
		cfg,
		func_params,
		loop_.clone(),
		loop_bbs.clone(),
		exit_bb.unwrap(),
		exit_prev.unwrap(),
	);

	trace!("loop_info: \n{}", loop_info);

	if loop_info.instr_cnt > 100 {
		return;
	}
	if loop_info.loop_type == LoopType::IGNORE {
		return;
	}
	// 被展开次数
	let mut unroll_cnt = UNROLL_CNT;
	if loop_info.loop_type == LoopType::CONSTTERMINATED {
		// 总循环次数
		let mut full_cnt: i32;
		match loop_info.cond_op {
			CompOp::SLT => {
				full_cnt = (loop_info.end - loop_info.start + loop_info.step - 1)
					/ loop_info.step;
				if loop_info.start >= loop_info.end {
					full_cnt = 0;
				}
			}
			CompOp::SLE => {
				full_cnt =
					(loop_info.end - loop_info.start + loop_info.step) / loop_info.step;
				if loop_info.start > loop_info.end {
					full_cnt = 0;
				}
			}
			_ => unreachable!(),
		}
		if full_cnt <= 1 {
			return;
		}
		// 如果总循环次数比较小，或者该循环内指令的个数乘总循环次数比较小，就全部展开
		// 即，把循环体复制总循环次数次
		if (full_cnt < 30 || (full_cnt as i64) * loop_info.instr_cnt < 200) {
			unroll_cnt = full_cnt as usize;
		} else {
			// 不全展开的情况暂时没写 TODO
			return;
		}
	}
	// 确定展开这个循环
	loop_unroll_inner(func, temp_mgr, loop_, loop_info, loop_bbs, unroll_cnt);
}

// 1. 把控制循环进行与否相关的指令单独提出来, 塞入 entry 的后继中在循环内的那个基本块
// 2. 断开 backedge，把循环体复制 unroll_cnt 次，在 cfg 中两两之间相连，顺序执行，并插入一条循环变量增加 step 的语句，最后一个循环体与 entry 连一条 backedge
// 3. 检查是否全部展开，如果是，则使 backedge 指向 exit，丢弃 entry，仅保留循环变量的初始值

// after_entry 指 entry 块在循环内的那个直接后继块

#[allow(unused)]
fn loop_unroll_inner(
	func: &mut LlvmFunc,
	temp_mgr: &mut LlvmTempManager,
	loop_: LoopPtr,
	info: SimpleLoopInfo,
	loop_bbs: Vec<LlvmNode>,
	unroll_cnt: usize,
) {
	trace!(
		"loop unroll: type {}, unroll_cnt {}",
		info.loop_type,
		unroll_cnt
	);
	let cfg = &mut func.cfg;
	// 1. 把控制循环进行与否相关的指令单独提出来, 塞入 entry 的后继中在循环内的那个基本块(phi 指令除外，因为 phi 指令意味着该变量有初始值，并且每次循环后会发生变化)
	let entry = loop_.borrow().header.clone();
	let exit = info.exit.unwrap();

	let mut new_after_entry_instrs = entry
		.borrow()
		.instrs
		.iter()
		.filter(|&instr| {
			instr.get_write().is_none()
				|| (instr.get_write().unwrap() != info.cond_temp.clone().unwrap())
		})
		.cloned()
		.collect::<Vec<LlvmInstr>>();

	let after_entry =
		entry.borrow().succ.iter().find(|bb| **bb != exit).unwrap().clone();
	// after_entry.borrow_mut().phi_instrs.extend(new_after_entry_phi_instrs);
	new_after_entry_instrs.extend(after_entry.borrow().instrs.iter().cloned());
	let after_entry_instrs = after_entry.borrow().instrs.clone();
	after_entry.borrow_mut().instrs = new_after_entry_instrs;

	// trace!("cfg: \n{}", cfg);

	// 2. 断开 backedge，把循环体复制 unroll_cnt 次，在 cfg 中两两之间相连，顺序执行，最后一个循环体与 entry 连一条 backedge
	unlink_node(info.backedge_start.as_ref().unwrap(), &entry);

	let mut next_bb_id = func.total + 1;
	let mut bb_map: HashMap<i32, LlvmNode> = HashMap::new();
	let mut cur_backedge_start = info.backedge_start.clone().unwrap();
	let mut cur_backedge_start_pos =
		cfg.blocks.iter().position(|bb| *bb == cur_backedge_start).unwrap();

	trace!(
		"cur_backedge_start: {}",
		cur_backedge_start.borrow().label()
	);

	// 维护临时变量的映射关系
	let mut temp_map: HashMap<LlvmTemp, LlvmTemp> = HashMap::new();
	// k 是 phi 指令定义的变量，v 是 phi 指令更新的目标，它应当静态不变
	let mut static_phi_temp_at_entry_map = HashMap::new();
	// k 是 phi 指令定义的变量，v 是上一次复制循环体中 k 映射到的变量，每复制一次，v 都要更新一次
	let mut dynamic_phi_temp_at_entry_map = HashMap::new();

	let entry_phi_defs = entry
		.borrow()
		.phi_instrs
		.iter()
		.map(|instr| instr.target.clone())
		.collect::<Vec<LlvmTemp>>();
	for instr in entry.borrow_mut().phi_instrs.iter_mut() {
		for (v, l) in instr.source.iter_mut() {
			if *l == cur_backedge_start.borrow().label() {
				static_phi_temp_at_entry_map
					.insert(instr.target.clone(), v.unwrap_temp().unwrap());
				dynamic_phi_temp_at_entry_map
					.insert(instr.target.clone(), instr.target.clone());
				break;
			}
		}
	}

	assert!(entry_phi_defs.len() == static_phi_temp_at_entry_map.len());

	for bb in loop_bbs.iter() {
		if *bb == entry {
			continue;
		}
		for instr in bb.borrow().instrs.iter() {
			if let Some(write) = instr.get_write() {
				trace!("temp_map insert: {}", write);
				temp_map.insert(write.clone(), write.clone());
			}
		}
		for instr in bb.borrow().phi_instrs.iter() {
			temp_map.insert(instr.target.clone(), instr.target.clone());
		}
	}

	for i in 0..unroll_cnt - 1 {
		bb_map.clear();
		// 复制块
		for bb in loop_bbs.iter() {
			if *bb == entry {
				continue;
			}
			let mut new_bb = bb.borrow().clone();
			new_bb.id = next_bb_id;
			trace!("bb {} to newbb {}", bb.borrow().label(), new_bb.label());
			new_bb.prev.clear();
			new_bb.succ.clear();
			new_bb.clear_data_flow();
			new_bb.kills.clear();
			new_bb.phi_defs.clear();
			bb_map.insert(bb.borrow().id, Rc::new(RefCell::new(new_bb)));

			cfg.blocks.insert(
				cur_backedge_start_pos + 1,
				bb_map.get(&bb.borrow().id).unwrap().clone(),
			);
			cur_backedge_start_pos += 1;

			next_bb_id += 1;
		}
		// 复制块间的连接关系
		for bb in loop_bbs.iter() {
			if *bb == entry {
				continue;
			}
			let new_bb = bb_map.get(&bb.borrow().id).unwrap();

			trace!("connecting new_bb {}", new_bb.borrow().label());

			assert!(new_bb.borrow().prev.is_empty());
			let mut prev_label_map = HashMap::new();
			new_bb.borrow_mut().prev = bb
				.borrow()
				.prev
				.iter()
				.map(|prev| {
					assert!(loop_bbs.contains(prev));
					let new_prev = if *prev == entry {
						assert!(new_bb.borrow().phi_instrs.is_empty());
						cur_backedge_start.clone()
					} else {
						bb_map.get(&prev.borrow().id).unwrap().clone()
					};
					prev_label_map
						.insert(prev.borrow().label(), new_prev.borrow().label());
					new_prev
				})
				.collect();
			new_bb
				.borrow_mut()
				.phi_instrs
				.iter_mut()
				.for_each(|instr| instr.map_label(&prev_label_map));

			assert!(new_bb.borrow().succ.is_empty());
			if bb == info.backedge_start.as_ref().unwrap() {
				continue;
			}

			let mut label_map = HashMap::new();
			new_bb.borrow_mut().succ = bb
				.borrow()
				.succ
				.iter()
				.map(|succ| {
					trace!(
						"bb: {}, succ: {}",
						bb.borrow().label(),
						succ.borrow().label()
					);
					assert!(loop_bbs.contains(succ));
					let new_succ = bb_map.get(&succ.borrow().id).unwrap().clone();
					label_map.insert(succ.borrow().label(), new_succ.borrow().label());
					new_succ
				})
				.collect();
			new_bb.borrow_mut().jump_instr.as_mut().unwrap().map_label(&label_map);
		}

		{
			let new_bb = bb_map.get(&after_entry.borrow().id).unwrap().clone();
			cur_backedge_start.borrow_mut().succ.push(new_bb.clone());
			cur_backedge_start.borrow_mut().jump_instr =
				Some(JumpInstr::new(new_bb.borrow().label()));
		}

		assert!(cur_backedge_start.borrow().succ.len() == 1);

		cur_backedge_start = bb_map
			.get(&info.backedge_start.as_ref().unwrap().borrow().id)
			.unwrap()
			.clone();

		let mut dynamic_copy = dynamic_phi_temp_at_entry_map.clone();
		// 假设 %2 是 entry 中的一个 phi temp，在 entry 中映射到 %4，那么在展开的每一轮中，%2 应当被映射为上一轮中 %4 被映射到的值
		for (k, v) in static_phi_temp_at_entry_map.iter_mut() {
			assert!(!temp_map.contains_key(k));
			let new_v = if let Some(s) = temp_map.get(v) {
				s.clone()
			} else {
				dynamic_phi_temp_at_entry_map.get(v).unwrap().clone()
			};
			dynamic_copy.entry(k.clone()).and_modify(|v| *v = new_v.clone());
			trace!("map {} to {}", k, new_v);
		}
		dynamic_phi_temp_at_entry_map.clone_from(&dynamic_copy);
		drop(dynamic_copy);

		for (k, v) in temp_map.iter_mut() {
			assert!(!static_phi_temp_at_entry_map.contains_key(k));
			*v = temp_mgr.new_temp(k.var_type, false);
			trace!("map {} to {}", k, v);
		}

		for bb in loop_bbs.iter() {
			if *bb == entry {
				continue;
			}
			let new_bb = bb_map.get(&bb.borrow().id).unwrap().clone();
			for instr in new_bb.borrow_mut().phi_instrs.iter_mut() {
				instr.map_all_temp(&temp_map);
				instr.map_all_temp(&dynamic_phi_temp_at_entry_map);
			}
			for instr in new_bb.borrow_mut().instrs.iter_mut() {
				instr.map_all_temp(&temp_map);
				instr.map_all_temp(&dynamic_phi_temp_at_entry_map);
			}
			for instr in new_bb.borrow_mut().jump_instr.iter_mut() {
				instr.map_all_temp(&temp_map);
				instr.map_all_temp(&dynamic_phi_temp_at_entry_map);
			}
		}
	}

	after_entry.borrow_mut().instrs = after_entry_instrs;

	// 3. 检查是否全部展开，如果是，则使 backedge 指向 exit，丢弃 entry，仅保留循环变量的初始值
	// 暂时没检查
	cur_backedge_start.borrow_mut().succ.push(entry.clone());
	cur_backedge_start.borrow_mut().jump_instr =
		Some(JumpInstr::new(entry.borrow().label()));

	entry.borrow_mut().prev.push(cur_backedge_start.clone());

	for phi in entry.borrow_mut().phi_instrs.iter_mut() {
		let source_temp_to_change =
			static_phi_temp_at_entry_map.get(&phi.target).unwrap();
		for (v, l) in phi.source.iter_mut() {
			if let Value::Temp(t) = v {
				if t == source_temp_to_change {
					let new_t = if let Some(s) = temp_map.get(t) {
						s.clone()
					} else {
						dynamic_phi_temp_at_entry_map.get(t).unwrap().clone()
					};
					*v = Value::Temp(new_t).clone();
					*l = cur_backedge_start.borrow().label();
					break;
				}
			}
		}
	}

	func.total = next_bb_id;
}
