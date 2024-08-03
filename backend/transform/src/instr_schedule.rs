use std::{
	cmp::min,
	collections::{HashMap, VecDeque},
};

use crate::{
	instrdag::{postprocess_call, InstrDag},
	Liveliness,
};
use instruction::{
	riscv::value::RiscvTemp::{self},
	RiscvInstrSet,
};
use utils::{
	SysycError, ADD_ALLOCATABLES, BFS_STATE_THRESHOLD, LIVE_THROUGH, NEAR_END,
	REDUCE_LIVE, REDUCE_SUB, SUM_MIN_RATIO,
};

// 当前惩罚策略：在指令为 instrs 的情况下，在运行每一条指令期间活跃的最大寄存器数目
// 接受参数：dag:初始图，instrs:当前的指令序列，基本块内 SSA
fn punishment(
	dag: InstrDag,
	state: &State,
	instr_id: usize,
	my_reads: Vec<RiscvTemp>,
	my_writes: Vec<RiscvTemp>,
) -> i32 {
	let mut score = 0;
	for i in my_reads.iter() {
		if state.liveliness_map.get(i).unwrap().use_num == 1
			&& !state.liveliness_map.get(i).unwrap().is_liveout
		{
			score -= 1;
		}
	}
	for i in my_writes.iter() {
		if !state.liveliness_map.get(i).unwrap().is_livein {
			score += 1;
		}
	}
	// 判断选择这条指令之后，有多少节点可以变成可调度节点
	let new_allocatables = dag.nodes[instr_id]
		.borrow()
		.succ
		.iter()
		.filter(|x| state.indegs[&x.borrow().id] == 1)
		.count();
	let alloc_score = -(new_allocatables as i32) * ADD_ALLOCATABLES;
	// 判断使得寄存器生命周期尽快结束的惩罚，一方面可以判断 read/write 的寄存器的尽快结束之和，另一方面可以判断 read/write 的寄存器最小离结束的次数,这一段 read 和 write 都是加，是没问题的
	// 思考 live_through 这个参数定义了没用，该怎么用上
	let mut sum_uses: usize = my_reads
		.iter()
		.map(|x| {
			if state.liveliness_map.get(x).unwrap().is_liveout {
				state.liveliness_map.get(x).unwrap().use_num + LIVE_THROUGH
			} else {
				state.liveliness_map.get(x).unwrap().use_num
			}
		})
		.sum();
	let mut min_uses: usize = my_reads
		.iter()
		.map(|x| {
			if state.liveliness_map.get(x).unwrap().is_liveout {
				state.liveliness_map.get(x).unwrap().use_num + LIVE_THROUGH
			} else {
				state.liveliness_map.get(x).unwrap().use_num
			}
		})
		.min()
		.unwrap_or(0);
	sum_uses += my_writes
		.iter()
		.map(|x| {
			if state.liveliness_map.get(x).unwrap().is_livein {
				state.liveliness_map.get(x).unwrap().use_num + LIVE_THROUGH
			} else {
				state.liveliness_map.get(x).unwrap().use_num
			}
		})
		.sum::<usize>();
	min_uses = min(
		my_writes
			.iter()
			.map(|x| {
				if state.liveliness_map.get(x).unwrap().is_livein {
					state.liveliness_map.get(x).unwrap().use_num + LIVE_THROUGH
				} else {
					state.liveliness_map.get(x).unwrap().use_num
				}
			})
			.min()
			.unwrap_or(0),
		min_uses,
	);
	let mut end_live_score = (sum_uses as i32) * SUM_MIN_RATIO;
	end_live_score += min_uses as i32;
	// 判断对后继的影响
	let mut succ_sum = 0;
	let mut succ_min = 0;
	for i in dag.nodes[instr_id].borrow().succ.iter() {
		let mut my_succ_reads = Vec::new();
		if i.borrow().instr.is_call() {
			my_succ_reads.clone_from(&dag.call_reads[state.call_ids.len()]);
		} else {
			my_succ_reads.clone_from(&i.borrow().instr.get_riscv_read());
		}
		succ_sum += my_succ_reads
			.iter()
			.map(|x| {
				if state.liveliness_map.get(x).unwrap().is_liveout {
					state.liveliness_map.get(x).unwrap().use_num + LIVE_THROUGH
				} else {
					state.liveliness_map.get(x).unwrap().use_num
				}
			})
			.sum::<usize>();
		succ_min = min(
			my_succ_reads
				.iter()
				.map(|x| state.liveliness_map.get(x).unwrap().use_num)
				.min()
				.unwrap_or(0),
			succ_min,
		);
		// 对 write 寄存器的情况考虑如上
		let mut my_succ_writes = Vec::new();
		if i.borrow().instr.is_call() {
			my_succ_writes = if let Some(tmp) = dag.call_writes[state.call_ids.len()]
			{
				vec![tmp]
			} else {
				Vec::new()
			};
		} else {
			my_succ_writes.clone_from(&i.borrow().instr.get_riscv_write());
		}
		succ_sum += my_succ_writes
			.iter()
			.map(|x| {
				if state.liveliness_map.get(x).unwrap().is_livein {
					state.liveliness_map.get(x).unwrap().use_num + LIVE_THROUGH
				} else {
					state.liveliness_map.get(x).unwrap().use_num
				}
			})
			.sum::<usize>();
		succ_min = min(
			my_succ_writes
				.iter()
				.map(|x| {
					if state.liveliness_map.get(x).unwrap().is_livein {
						state.liveliness_map.get(x).unwrap().use_num + LIVE_THROUGH
					} else {
						state.liveliness_map.get(x).unwrap().use_num
					}
				})
				.min()
				.unwrap_or(0),
			succ_min,
		);
	}
	let mut succ_score = (succ_sum as i32) * SUM_MIN_RATIO;
	succ_score += succ_min as i32;
	score = score * REDUCE_LIVE
		+ alloc_score * ADD_ALLOCATABLES
		+ end_live_score * NEAR_END
		+ succ_score * REDUCE_SUB;
	score
}
#[derive(Clone)]
struct State {
	instrs: RiscvInstrSet,
	score: i32,
	indegs: HashMap<usize, usize>, // 把节点的 id 映射到入度
	liveliness_map: HashMap<RiscvTemp, Liveliness>,
	call_ids: Vec<usize>,
}
// todo 降常数复杂度（只对前面的若干个去 clone liveliness_map 和 indegs），问中端友友纯函数怎么判断，改纯函数的 InstrDag
// 咱想想怎么设计：改动：
// 1. 先不去 clone state，对于每个可以分配的 instruction 把 instr 先 push 再 pop 最后把 pop_front 得到的 State 再 push 回去
// 2. 每一步的计算保留以下4个参数：total_punishment,state_idx,node_id,my_reads 最后根据 total_punishment 排序并且把前 BFS_STATE_THRESHOLD 给 push 进去
pub fn instr_schedule_by_dag(
	dag: InstrDag,
	liveliness_map: HashMap<RiscvTemp, Liveliness>,
) -> Result<RiscvInstrSet, SysycError> {
	// println!("{}",dag);
	let mut states = VecDeque::new();
	// calculate indegs
	let mut indegs = HashMap::new();
	for node in dag.nodes.iter() {
		indegs.insert(node.borrow().id, node.borrow().in_deg);
	}
	states.push_back(State {
		instrs: Vec::new(),
		score: 0,
		indegs: indegs.clone(),
		liveliness_map,
		call_ids: Vec::new(),
	});
	let depth = dag.nodes.len(); // bfs 深度已知，是所需要调度的指令总数
	for _i in 0..depth {
		let real_cnt = states.len();
		let mut keeps = Vec::new();
		for j in 0..real_cnt {
			let mut state = states.pop_front().unwrap();
			let allocatables: Vec<_> = state
				.indegs
				.iter()
				.filter(|(_k, v)| **v == 0)
				.map(|(k, _)| *k)
				.collect();
			// println!("allocatables: {:?} _i: {:?} _j: {:?} ", allocatables,_i,_j);
			// println!("state instrs:");
			// for i in state.instrs.iter() {
			// 	println!("{}", i);
			// }
			for i in allocatables.iter() {
				//let mut new_state = state.clone();
				state.instrs.push(dag.nodes[*i].borrow().instr.clone());
				// get riscv reads and writes
				let mut my_reads = Vec::new();
				let mut my_writes = Vec::new();
				if dag.nodes[*i].borrow().instr.is_call() {
					//check state's call_id length
					my_reads.clone_from(&dag.call_reads[state.call_ids.len()]);
					my_writes = if let Some(tmp) = dag.call_writes[state.call_ids.len()] {
						vec![tmp]
					} else {
						Vec::new()
					};
				} else {
					my_reads.clone_from(&dag.nodes[*i].borrow().instr.get_riscv_read());
					my_writes.clone_from(&dag.nodes[*i].borrow().instr.get_riscv_write());
				}
				let score = state.score
					+ punishment(
						dag.clone(),
						&state,
						*i,
						my_reads.clone(),
						my_writes.clone(),
					);
				keeps.push((j, *i, score));
				state.instrs.pop();
			}
			states.push_back(state);
		}
		if keeps.len() > BFS_STATE_THRESHOLD {
			keeps.sort_by(|a, b| a.2.cmp(&b.2));
			keeps.truncate(BFS_STATE_THRESHOLD);
		}
		for i in 0..real_cnt {
			// iterate the keeps
			let cnts: Vec<_> = keeps.iter().filter(|x| x.0 == i).copied().collect();
			if cnts.is_empty() {
				states.pop_front();
			} else if cnts.len() == 1 {
				let mut state = states.pop_front().unwrap();
				state.instrs.push(dag.nodes[cnts[0].1].borrow().instr.clone());
				if dag.nodes[cnts[0].1].borrow().instr.is_call() {
					state.call_ids.push(cnts[0].1);
				}
				// calc my_reads
				let mut my_reads = Vec::new();
				if state.instrs.last().unwrap().is_call() {
					my_reads.clone_from(&dag.call_reads[state.call_ids.len() - 1]);
				} else {
					my_reads
						.clone_from(&dag.nodes[cnts[0].1].borrow().instr.get_riscv_read());
				}
				// decl the use in new_state's liveliness_map
				for i in my_reads.iter() {
					state.liveliness_map.get_mut(i).unwrap().use_num -= 1;
				}
				state.indegs.remove(&cnts[0].1);
				for succ in dag.nodes[cnts[0].1].borrow().succ.iter() {
					let mut new_indeg = state.indegs.clone();
					new_indeg.insert(
						succ.borrow().id,
						new_indeg.get(&succ.borrow().id).unwrap() - 1,
					);
					state.indegs = new_indeg;
				}
				states.push_back(state);
			} else {
				let mut state = states.pop_front().unwrap();
				for j in cnts.iter().take(cnts.len() - 1) {
					let mut new_state = state.clone();
					new_state.instrs.push(dag.nodes[j.1].borrow().instr.clone());
					if dag.nodes[j.1].borrow().instr.is_call() {
						new_state.call_ids.push(j.1);
					}
					// calc my_reads
					let mut my_reads = Vec::new();
					if new_state.instrs.last().unwrap().is_call() {
						my_reads.clone_from(&dag.call_reads[new_state.call_ids.len() - 1]);
					} else {
						my_reads
							.clone_from(&dag.nodes[j.1].borrow().instr.get_riscv_read());
					}
					// decl the use in new_state's liveliness_map
					for i in my_reads.iter() {
						new_state.liveliness_map.get_mut(i).unwrap().use_num -= 1;
					}
					new_state.indegs.remove(&j.1);
					for succ in dag.nodes[j.1].borrow().succ.iter() {
						let mut new_indeg = new_state.indegs.clone();
						new_indeg.insert(
							succ.borrow().id,
							new_indeg.get(&succ.borrow().id).unwrap() - 1,
						);
						new_state.indegs = new_indeg;
					}
					states.push_back(new_state);
				}
				// 最后一次不 clone 了
				state
					.instrs
					.push(dag.nodes[cnts[cnts.len() - 1].1].borrow().instr.clone());
				if dag.nodes[cnts[cnts.len() - 1].1].borrow().instr.is_call() {
					state.call_ids.push(cnts[cnts.len() - 1].1);
				}
				// calc my_reads
				let mut my_reads = Vec::new();
				if state.instrs.last().unwrap().is_call() {
					my_reads.clone_from(&dag.call_reads[state.call_ids.len() - 1]);
				} else {
					my_reads.clone_from(
						&dag.nodes[cnts[cnts.len() - 1].1].borrow().instr.get_riscv_read(),
					);
				}
				// decl the use in new_state's liveliness_map
				for i in my_reads.iter() {
					state.liveliness_map.get_mut(i).unwrap().use_num -= 1;
				}
				state.indegs.remove(&cnts[cnts.len() - 1].1);
				for succ in dag.nodes[cnts[cnts.len() - 1].1].borrow().succ.iter() {
					let mut new_indeg = state.indegs.clone();
					new_indeg.insert(
						succ.borrow().id,
						new_indeg.get(&succ.borrow().id).unwrap() - 1,
					);
					state.indegs = new_indeg;
				}
				states.push_back(state);
			}
		}
	}
	// for i in states.iter() {
	// 	println!("final state instructions:");
	// 	for j in i.instrs.iter() {
	// 		println!("{}", j);
	// 	}
	// }
	let final_state = states.pop_front().unwrap();
	// println!("final state instructions:");
	// for i in final_state.instrs.iter() {
	// 	println!("{}", i);
	// }
	Ok(postprocess_call(
		final_state.instrs,
		&mut dag.call_related.clone(), // 是我call的顺序可能会调换，post_process 的时候和原本push进去的顺序不一致
		dag.branch.clone(),
		&mut final_state.call_ids.clone(),
	))
}
