use std::{cell::RefCell, collections::HashMap, rc::Rc, vec};

use llvm::{LlvmInstrTrait, LlvmTemp};
use rrvm::{cfg::unlink_node, rrvm_loop::LoopPtr};
use utils::{MAX_UNROLL_INSTR_CNT, MAX_UNROLL_TOTAL_INSTR_CNT};

use crate::loops::loopinfo::LoopInfo;

use super::LoopUnroll;

impl<'a> LoopUnroll<'a> {
	// 成功展开返回 true, 否则返回 false
	pub fn unroll_one_loop(&mut self, loop_: LoopPtr, info: LoopInfo) -> bool {
		if let Some(loop_cnt) = info.has_const_loop_cnt() {
			if loop_cnt <= 1 {
				return false;
			}
			let loop_cnt = loop_cnt as usize;
			let instr_cnt = loop_.borrow().instr_cnt(&self.loopdata.loop_map);
			if instr_cnt > MAX_UNROLL_INSTR_CNT {
				return false;
			}
			if instr_cnt * loop_cnt > MAX_UNROLL_TOTAL_INSTR_CNT {
				return false;
			}
			eprintln!(
				"Unroll loop: {} cnt: {}",
				loop_.borrow().header.borrow().label(),
				loop_cnt
			);
			self.loop_unroll_inner(&info, loop_, loop_cnt, true)
		} else {
			false
		}
	}
	/// 1. 把 backedge 全部断开
	/// 2. 复制循环体，将 backedge 指向复制出来的 header
	/// 3. 映射新 temp
	/// 4. 重复 loop_cnt 次, 最后一次的 backedge 指向原来的 header
	/// 5. 如果为全展开，则断开 header 到 exit 的边，将最后一次的 backedge 指向 exit
	/// 返回成功与否
	pub fn loop_unroll_inner(
		&mut self,
		info: &LoopInfo,
		loop_: LoopPtr,
		unroll_cnt: usize,
		is_full_unroll: bool,
	) -> bool {
		// 断开全部 backedge
		let header = loop_.borrow().header.clone();
		let latches = loop_.borrow().get_loop_latches(&self.loopdata.loop_map);
		latches.iter().for_each(|latch| {
			unlink_node(latch, &header);
		});

		// 断开 header 与 exit 的连接
		unlink_node(&header, &info.single_exit);
		header.borrow_mut().gen_jump(llvm::VarType::Void);

		let mut next_bb_id = self.func.total + 1;

		// 维护临时变量的映射关系
		// 每复制好一次循环后，其中的值为上一轮循环所映射到的值
		let mut temp_map: HashMap<LlvmTemp, LlvmTemp> = HashMap::new();
		let loop_bbs = loop_.borrow().blocks(&self.loopdata.loop_map);
		for bb in loop_bbs.iter() {
			for instr in bb.borrow().instrs.iter() {
				if let Some(write) = instr.get_write() {
					temp_map.insert(write.clone(), write.clone());
				}
			}
			for instr in bb.borrow().phi_instrs.iter() {
				temp_map.insert(instr.target.clone(), instr.target.clone());
			}
		}

		// 找到新 block 应该被插入的位置
		// 即最后一个 latch 的位置
		let mut pos_to_insert = self.func.cfg.blocks.len();
		for i in (0..self.func.cfg.blocks.len()).rev() {
			if latches.contains(&self.func.cfg.blocks[i]) {
				pos_to_insert = i + 1;
				break;
			}
		}

		// 用于寻找新 header 的前驱
		let mut latches_map = latches
			.iter()
			.map(|latch| (latch.borrow().id, latch.clone()))
			.collect::<HashMap<_, _>>();

		// 暂时从 header 中把 phi 的初始值除去，便于接下来复制
		let mut phi_initial_value = HashMap::new();
		for phi in header.borrow_mut().phi_instrs.iter_mut() {
			let initial_value = phi
				.source
				.iter()
				.find(|(_, label)| *label == info.preheader.borrow().label())
				.unwrap();
			phi_initial_value.insert(phi.target.clone(), initial_value.0.clone());
			phi.source.retain(|(_, label)| *label != info.preheader.borrow().label());
		}

		let mut bb_to_insert = vec![];

		for _ in 0..unroll_cnt - 1 {
			// 复制块
			let bb_map = loop_bbs
				.iter()
				.map(|bb| {
					let mut new_bb = bb.borrow().clone();
					new_bb.clear();
					new_bb.clear_data_flow();
					new_bb.kills.clear();
					new_bb.id = next_bb_id;
					next_bb_id += 1;
					(bb.borrow().id, Rc::new(RefCell::new(new_bb)))
				})
				.collect::<HashMap<_, _>>();
			// 复制块间的前驱关系，映射块中的变量
			// 新块的前驱后继是旧块的前驱后继映射到的新块
			// 特别地，新 header 的前驱是上一次 latches 映射到的块
			// 上一次 latch 的后继是新 header
			for bb in loop_bbs.iter() {
				let is_mapping_header = bb.borrow().id == header.borrow().id;
				let is_mapping_latch = latches.contains(bb);
				let new_bb = bb_map[&bb.borrow().id].clone();
				bb_to_insert.push(new_bb.clone());
				// 维护前驱关系
				assert!(new_bb.borrow().prev.is_empty());
				let mut prev_label_map = HashMap::new();
				new_bb.borrow_mut().prev = if is_mapping_header {
					latches
						.iter()
						.map(|latch| {
							let new_latch =
								latches_map.get(&latch.borrow().id).unwrap().clone();
							prev_label_map
								.insert(latch.borrow().label(), new_latch.borrow().label());
							new_latch
						})
						.collect::<Vec<_>>()
				} else {
					bb.borrow()
						.prev
						.iter()
						.map(|prev| {
							let new_prev = bb_map.get(&prev.borrow().id).unwrap().clone();
							prev_label_map
								.insert(prev.borrow().label(), new_prev.borrow().label());
							new_prev
						})
						.collect::<Vec<_>>()
				};

				// 维护后继关系
				let mut succ_label_map = HashMap::new();
				assert!(new_bb.borrow().succ.is_empty());
				if !latches.contains(bb) {
					new_bb.borrow_mut().succ = bb
						.borrow()
						.succ
						.iter()
						.map(|succ| {
							let new_succ = bb_map.get(&succ.borrow().id).unwrap().clone();
							succ_label_map
								.insert(succ.borrow().label(), new_succ.borrow().label());
							new_succ
						})
						.collect::<Vec<_>>();
				} else {
					// 在此处完成对上一轮 latch 的后继的映射
					let new_header = bb_map[&header.borrow().id].clone();
					let old_latch = latches_map.get(&bb.borrow().id).unwrap().clone();
					old_latch.borrow_mut().succ = vec![new_header.clone()];
					assert!(old_latch.borrow().jump_instr.is_none());
					old_latch.borrow_mut().gen_jump(llvm::VarType::Void);
				}

				// 维护 Temp 的映射关系
				if is_mapping_header {
					// header 中的 phi 的 use 需要使用上一轮映射的值, 但 target 不能变
					for phi in new_bb.borrow_mut().phi_instrs.iter_mut() {
						let old_target = phi.target.clone();
						phi.map_all_temp(&temp_map);
						phi.set_target(old_target);
					}
				}

				// 创造新 temp
				temp_map
					.iter_mut()
					.for_each(|(_, v)| *v = self.temp_mgr.new_temp(v.var_type, false));
				// 映射新 temp
				if is_mapping_header {
					// header 中的 phi 只需要映射 target
					for phi in new_bb.borrow_mut().phi_instrs.iter_mut() {
						let old_target = phi.target.clone();
						phi.set_target(temp_map[&old_target].clone());
						phi.map_label(&prev_label_map);
					}
				} else {
					for phi in new_bb.borrow_mut().phi_instrs.iter_mut() {
						phi.map_all_temp(&temp_map);
						phi.map_label(&prev_label_map);
					}
				}
				for instr in new_bb.borrow_mut().instrs.iter_mut() {
					instr.map_all_temp(&temp_map);
				}
				if is_mapping_latch {
					latches_map.insert(bb.borrow().id, new_bb.clone());
				} else {
					for jump in new_bb.borrow_mut().jump_instr.iter_mut() {
						jump.map_all_temp(&temp_map);
						jump.map_label(&succ_label_map);
					}
				}
			}
		}

		self.func.cfg.blocks.splice(pos_to_insert..pos_to_insert, bb_to_insert);
		// 如果是全部展开则
		// latch 指向 exit
		// 从 header 中把 phi 语句都薅过来，target 不变，修改 sources，放到 exit 的 phi_instrs 中
		// header 中原有的 phi 的 target 都变成从 preheader 来的初始值
		// 从 header 中把其余 instr 都薅过来，target map 成新的，放到 exit 的 instrs 的前面
		// 否则
		// latch 指向 header
		// header 中 phi 的 sources 被修改
		if is_full_unroll {
			let mut label_map = HashMap::new();
			for latch in latches.iter() {
				let mapped_latch = latches_map[&latch.borrow().id].clone();
				mapped_latch.borrow_mut().succ = vec![info.single_exit.clone()];
				assert!(mapped_latch.borrow().jump_instr.is_none());
				mapped_latch.borrow_mut().gen_jump(llvm::VarType::Void);
				label_map.insert(latch.borrow().label(), mapped_latch.borrow().label());
			}
			let mut phis = header.borrow_mut().phi_instrs.clone();
			for phi in phis.iter_mut() {
				phi.map_label(&label_map);
				phi.map_all_temp(&temp_map);
			}
			assert!(info.single_exit.borrow().phi_instrs.is_empty());
			info.single_exit.borrow_mut().phi_instrs = phis;
			let mut new_target_map = HashMap::new();
			for instr in header.borrow_mut().instrs.iter_mut() {
				if let Some(write) = instr.get_write() {
					let new_write = self.temp_mgr.new_temp(write.var_type, false);
					new_target_map.insert(write.clone(), new_write.clone());
				}
				instr.map_temp(&phi_initial_value);
			}
			let mut new_instr = header.borrow().instrs.clone();
			new_instr
				.iter_mut()
				.for_each(|instr| instr.map_all_temp(&new_target_map));
			new_instr.append(&mut info.single_exit.borrow_mut().instrs);
			info.single_exit.borrow_mut().instrs = new_instr;
		} else {
			let mut label_map = HashMap::new();
			for latch in latches.iter() {
				let mapped_latch = latches_map[&latch.borrow().id].clone();
				mapped_latch.borrow_mut().succ = vec![info.header.clone()];
				assert!(mapped_latch.borrow().jump_instr.is_none());
				mapped_latch.borrow_mut().gen_jump(llvm::VarType::Void);
				label_map.insert(latch.borrow().label(), mapped_latch.borrow().label());
			}
			let mut phis = header.borrow_mut().phi_instrs.clone();
			for phi in phis.iter_mut() {
				phi.map_label(&label_map);
				phi.map_all_temp(&temp_map);
			}
		}
		false
	}
}
