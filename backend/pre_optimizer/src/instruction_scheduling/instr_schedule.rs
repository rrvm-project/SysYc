use std::{
	cmp::{max, min},
	collections::{HashMap, VecDeque},
	fmt::Display,
};

use crate::instruction_scheduling::{
	instrdag::{postprocess_call, InstrDag},
	Liveliness,
};
use instruction::{
	riscv::{
		prelude::RiscvInstrTrait,
		riscvinstr::RiscvInstr,
		value::RiscvTemp::{self},
	},
	RiscvInstrSet,
};
use utils::{
	SysycError, ADD_ALLOCATABLES, BFS_STATE_THRESHOLD, HARDWARE_PIPELINE_PARAM,
	LIVE_THROUGH, NEAR_END, REDUCE_LIVE, REDUCE_SUB, SOFTWARE_PIPELINE_PARAM,
	SUM_MIN_RATIO,
};

#[derive(Clone, PartialEq, Eq, Copy, Debug)]
enum AluKind {
	Mem,
	Normal,
	Branch,
	Float,
	MulDiv,
}
#[derive(Clone, Copy, Debug)]
pub struct Alu {
	kind: AluKind,
	complete_cycle: usize,
	is_fdiv: bool,
}
impl Alu {
	fn new(kind: AluKind) -> Self {
		Self {
			kind,
			complete_cycle: 0, // 开区间
			is_fdiv: false,
		}
	}
}
fn get_alukind(instr: &RiscvInstr) -> AluKind {
	let v = instr.get_rtn_array();
	if v[0] != 0 {
		AluKind::Mem
	} else if v[1] != 0 {
		AluKind::Branch
	} else if v[2] != 0 {
		AluKind::MulDiv
	} else if v[3] != 0 {
		AluKind::Float
	} else {
		AluKind::Normal
	}
}
// 当前惩罚策略：在指令为 instrs 的情况下，在运行每一条指令期间活跃的最大寄存器数目
// 接受参数：dag:初始图，instrs:当前的指令序列，基本块内 SSA
// 实现硬件流水线的时候，要多返回一个 flight_time_increment
fn punishment(
	dag: &InstrDag,
	state: &State,
	instr_id: usize,
	my_reads: Vec<RiscvTemp>,
	my_writes: Vec<RiscvTemp>,
) -> (i32, usize, usize, Alu) {
	let instr = state.instrs.last().unwrap();
	let mut score = 0;
	// 软件流水线的惩罚
	score +=
		(dag.nodes[instr_id].borrow().to_end as i32) * SOFTWARE_PIPELINE_PARAM;
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
		let my_succ_reads = {
			if i.borrow().instr.is_call() {
				dag.call_reads[state.call_ids.len()].clone()
			} else {
				i.borrow().instr.get_riscv_read().clone()
			}
		};
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
		let my_succ_writes = {
			if i.borrow().instr.is_call() {
				if let Some(tmp) = dag.call_writes[state.call_ids.len()] {
					vec![tmp]
				} else {
					Vec::new()
				}
			} else {
				i.borrow().instr.get_riscv_write().clone()
			}
		};
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
	// 算硬件流水线的惩罚
	let mut flight_time_incre = 1;
	let ready_time = state.flight_time + flight_time_incre;
	let mut flight_idx = 0;
	let mut flight_unit = Alu::new(AluKind::Normal);
	let old_max = state.alus.iter().map(|x| x.complete_cycle).max().unwrap_or(0);
	// 增量，认为第一条指令在时刻1发射
	if get_alukind(instr) != AluKind::Normal {
		for (idx, alu) in state.alus.iter().enumerate() {
			if get_alukind(instr) == alu.kind {
				if alu.complete_cycle > ready_time {
					// wait
					flight_time_incre = alu.complete_cycle - ready_time + 1;
				}
				flight_idx = idx;
				flight_unit = Alu::new(alu.kind);
				if instr.is_fdiv() {
					flight_unit.is_fdiv = true;
				}
				flight_unit.complete_cycle = state.flight_time
					+ flight_time_incre
					+ instr.get_rtn_array()[4] as usize;
				if instr.is_fdiv() && alu.is_fdiv {
					flight_unit.complete_cycle += utils::FDIV_WAIT;
				}
				break;
			}
		}
	} else {
		// 从 alus[4],alus[5] 拿出 complete_time 更小的来考虑
		flight_idx = if state.alus[4].complete_cycle < state.alus[5].complete_cycle
		{
			4
		} else {
			5
		};
		flight_unit = Alu::new(state.alus[flight_idx].kind);
		if state.alus[flight_idx].complete_cycle > ready_time {
			flight_time_incre =
				state.alus[flight_idx].complete_cycle - ready_time + 1;
		}
		flight_unit.complete_cycle =
			state.flight_time + flight_time_incre + instr.get_rtn_array()[4] as usize;
	}
	let time_incre = max(flight_unit.complete_cycle, old_max) - old_max;

	succ_score += succ_min as i32;
	score = score * REDUCE_LIVE
		+ alloc_score * ADD_ALLOCATABLES
		+ end_live_score * NEAR_END
		+ succ_score * REDUCE_SUB
		+ time_incre as i32 * HARDWARE_PIPELINE_PARAM;

	(score, flight_time_incre, flight_idx, flight_unit)
}
#[derive(Clone)]
struct State {
	instrs: RiscvInstrSet,
	score: i32,
	indegs: HashMap<usize, usize>, // 把节点的 id 映射到入度
	liveliness_map: HashMap<RiscvTemp, Liveliness>,
	call_ids: Vec<usize>,
	alus: [Alu; 6],
	flight_time: usize,
}
impl Display for State {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		writeln!(f, "State: ")?;
		for i in self.instrs.iter() {
			writeln!(f, "{}", i)?;
		}
		writeln!(f, "alus: ")?;
		for i in self.alus.iter() {
			write!(f, "{:?} ", i)?;
		}
		writeln!(f, "score: {} flight_time: {}", self.score, self.flight_time)?;
		Ok(())
	}
}
pub fn get_punishment_by_instrs(instr: &[Box<dyn RiscvInstrTrait>]) -> i32 {
	// 算出原始的 score
	//	按照上面的方法算硬件流水线
	let mut alus = [
		Alu::new(AluKind::Mem),
		Alu::new(AluKind::Branch),
		Alu::new(AluKind::MulDiv),
		Alu::new(AluKind::Float),
		Alu::new(AluKind::Normal),
		Alu::new(AluKind::Normal),
	];
	let mut flight_time = 0;
	for instr in instr.iter() {
		let mut flight_time_incre = 1;
		let ready_time = flight_time + flight_time_incre;
		if get_alukind(instr) != AluKind::Normal {
			for alu in alus.iter_mut() {
				if get_alukind(instr) == alu.kind {
					if alu.complete_cycle > ready_time {
						flight_time_incre = alu.complete_cycle - ready_time + 1;
					}
					if instr.is_fdiv() {
						alu.is_fdiv = true;
					}
					alu.complete_cycle =
						flight_time + flight_time_incre + instr.get_rtn_array()[4] as usize;
					if instr.is_fdiv() && alu.is_fdiv {
						alu.complete_cycle += utils::FDIV_WAIT;
					}
					break;
				}
			}
		} else {
			let flight_idx = if alus[4].complete_cycle < alus[5].complete_cycle {
				4
			} else {
				5
			};
			if alus[flight_idx].complete_cycle > ready_time {
				flight_time_incre = alus[flight_idx].complete_cycle - ready_time + 1;
			}
			alus[flight_idx].complete_cycle =
				flight_time + flight_time_incre + instr.get_rtn_array()[4] as usize;
		}
		flight_time += flight_time_incre;
	}
	let t = alus.iter().map(|x| x.complete_cycle).max().unwrap_or(0);
	t as i32 * HARDWARE_PIPELINE_PARAM
}
// 1. 先不去 clone state，对于每个可以分配的 instruction 把 instr 先 push 再 pop 最后把 pop_front 得到的 State 再 push 回去
// 2. 每一步的计算保留以下4个参数：total_punishment,state_idx,node_id,my_reads 最后根据 total_punishment 排序并且把前 BFS_STATE_THRESHOLD 给 push 进去
pub fn instr_schedule_by_dag(
	dag: InstrDag,
	liveliness_map: HashMap<RiscvTemp, Liveliness>,
) -> Result<RiscvInstrSet, SysycError> {
	// 计算原始 punishment
	let original_instrs: Vec<_> =
		dag.nodes.iter().rev().map(|x| x.borrow().instr.clone()).collect();
	let original_punishment = get_punishment_by_instrs(&original_instrs);
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
		alus: [
			Alu::new(AluKind::Mem),
			Alu::new(AluKind::Branch),
			Alu::new(AluKind::MulDiv),
			Alu::new(AluKind::Float),
			Alu::new(AluKind::Normal),
			Alu::new(AluKind::Normal),
		],
		flight_time: 0,
	});
	let depth = dag.nodes.len(); // bfs 深度已知，是所需要调度的指令总数
	for _ in 0..depth {
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
			for i in allocatables.iter() {
				state.instrs.push(dag.nodes[*i].borrow().instr.clone());
				// get riscv reads and writes
				let my_reads;
				let my_writes;
				if dag.nodes[*i].borrow().instr.is_call() {
					//check state's call_id length
					my_reads = dag.call_reads[state.call_ids.len()].clone();
					my_writes = if let Some(tmp) = dag.call_writes[state.call_ids.len()] {
						vec![tmp]
					} else {
						Vec::new()
					};
				} else {
					my_reads = dag.nodes[*i].borrow().instr.get_riscv_read().clone();
					my_writes = dag.nodes[*i].borrow().instr.get_riscv_write().clone();
				}
				let (punish, flight_time_incre, flight_idx, flight_unit) =
					punishment(&dag, &state, *i, my_reads.clone(), my_writes.clone());
				let score = state.score + punish;
				keeps.push((j, *i, score, flight_time_incre, flight_idx, flight_unit));
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
				let (
					_state_idx,
					instr_idx,
					score,
					flight_time_incre,
					flight_idx,
					flight_unit,
				) = cnts[0];
				state.instrs.push(dag.nodes[instr_idx].borrow().instr.clone());
				if dag.nodes[instr_idx].borrow().instr.is_call() {
					state.call_ids.push(instr_idx);
				}
				// calc my_reads
				let my_reads = {
					if state.instrs.last().unwrap().is_call() {
						dag.call_reads[state.call_ids.len() - 1].clone()
					} else {
						dag.nodes[instr_idx].borrow().instr.get_riscv_read().clone()
					}
				};
				// decl the use in new_state's liveliness_map
				for i in my_reads.iter() {
					state.liveliness_map.get_mut(i).unwrap().use_num -= 1;
				}
				state.indegs.remove(&instr_idx);
				for succ in dag.nodes[instr_idx].borrow().succ.iter() {
					let mut new_indeg = state.indegs.clone();
					new_indeg.insert(
						succ.borrow().id,
						new_indeg.get(&succ.borrow().id).unwrap() - 1,
					);
					state.indegs = new_indeg;
				}
				state.flight_time += flight_time_incre;
				state.alus[flight_idx] = flight_unit;
				state.score = score;
				states.push_back(state);
			} else {
				let mut state = states.pop_front().unwrap();
				for tuple in cnts.iter().take(cnts.len() - 1) {
					let mut new_state = state.clone();
					let (
						_state_idx,
						instr_idx,
						score,
						flight_time_incre,
						flight_idx,
						flight_unit,
					) = tuple;
					new_state.instrs.push(dag.nodes[*instr_idx].borrow().instr.clone());
					if dag.nodes[*instr_idx].borrow().instr.is_call() {
						new_state.call_ids.push(*instr_idx);
					}
					// calc my_reads
					let my_reads = {
						if new_state.instrs.last().unwrap().is_call() {
							dag.call_reads[new_state.call_ids.len() - 1].clone()
						} else {
							dag.nodes[*instr_idx].borrow().instr.get_riscv_read().clone()
						}
					};
					// decl the use in new_state's liveliness_map
					for i in my_reads.iter() {
						new_state.liveliness_map.get_mut(i).unwrap().use_num -= 1;
					}
					new_state.indegs.remove(instr_idx);
					for succ in dag.nodes[*instr_idx].borrow().succ.iter() {
						let mut new_indeg = new_state.indegs.clone();
						new_indeg.insert(
							succ.borrow().id,
							new_indeg.get(&succ.borrow().id).unwrap() - 1,
						);
						new_state.indegs = new_indeg;
					}
					new_state.flight_time += flight_time_incre;
					new_state.alus[*flight_idx] = *flight_unit;
					new_state.score = *score;
					states.push_back(new_state);
				}
				// 最后一次不 clone 了
				let (
					_state_idx,
					instr_idx,
					score,
					flight_time_incre,
					flight_idx,
					flight_unit,
				) = cnts.last().unwrap();
				state.instrs.push(dag.nodes[*instr_idx].borrow().instr.clone());
				if dag.nodes[*instr_idx].borrow().instr.is_call() {
					state.call_ids.push(*instr_idx);
				}
				// calc my_reads
				let my_reads = {
					if state.instrs.last().unwrap().is_call() {
						dag.call_reads[state.call_ids.len() - 1].clone()
					} else {
						dag.nodes[*instr_idx].borrow().instr.get_riscv_read().clone()
					}
				};
				// decl the use in new_state's liveliness_map
				for i in my_reads.iter() {
					state.liveliness_map.get_mut(i).unwrap().use_num -= 1;
				}
				state.indegs.remove(instr_idx);
				for succ in dag.nodes[*instr_idx].borrow().succ.iter() {
					let mut new_indeg = state.indegs.clone();
					new_indeg.insert(
						succ.borrow().id,
						new_indeg.get(&succ.borrow().id).unwrap() - 1,
					);
					state.indegs = new_indeg;
				}
				state.flight_time += flight_time_incre;
				state.alus[*flight_idx] = *flight_unit;
				state.score = *score;
				states.push_back(state);
			}
		}
	}
	// state 排序
	states.make_contiguous().sort_by(|a, b| a.score.cmp(&b.score));
	let mut final_state = states.pop_front().unwrap();
	if final_state.score >= original_punishment {
		final_state.instrs = original_instrs;
	}
	Ok(postprocess_call(
		final_state.instrs,
		&mut dag.call_related.clone(), // 是我call的顺序可能会调换，post_process 的时候和原本push进去的顺序不一致
		dag.branch.clone(),
		&mut final_state.call_ids.clone(),
		dag.li_ret.clone(),
	))
}
