use std::{
	cmp::min,
	collections::{HashMap, VecDeque},
};

use crate::{instrdag::InstrDag, Liveliness};
use instruction::{riscv::value::RiscvTemp, RiscvInstrSet};
use utils::{
	SysycError, ADD_ALLOCATABLES, BFS_STATE_THRESHOLD, NEAR_END, REDUCE_LIVE,
	REDUCE_SUB, SUM_MIN_RATIO,
};

// 当前惩罚策略：在指令为 instrs 的情况下，在运行每一条指令期间活跃的最大寄存器数目
// 接受参数：dag:在最初始图上删除和 instr 相关边所得到的图，instrs:当前的指令序列，基本块内 SSA
fn punishment(dag: InstrDag, state: &mut State, instr_id: usize) -> i32 {
	let instr = state.instrs.last().unwrap();
	let mut score = 0;
	for i in instr.get_riscv_read().iter() {
		if state.liveliness_map.get(i).unwrap().use_num == 1
			&& state.liveliness_map.get(i).unwrap().is_liveout == false
		{
			score -= 1;
		}
	}
	for i in instr.get_riscv_write().iter() {
		if state.liveliness_map.get(i).unwrap().is_livein == false {
			score -= 1;
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
	// 判断使得寄存器生命周期尽快结束的惩罚，一方面可以判断 read/write 的寄存器的尽快结束之和，另一方面可以判断 read/write 的寄存器最小离结束的次数
	let mut sum_uses: usize = dag.nodes[instr_id]
		.borrow()
		.instr
		.get_riscv_read()
		.iter()
		.map(|x| state.liveliness_map.get(x).unwrap().use_num)
		.sum();
	let mut min_uses: usize = dag.nodes[instr_id]
		.borrow()
		.instr
		.get_riscv_read()
		.iter()
		.map(|x| state.liveliness_map.get(x).unwrap().use_num)
		.min()
		.unwrap();
	sum_uses += dag.nodes[instr_id]
		.borrow()
		.instr
		.get_riscv_write()
		.iter()
		.map(|x| state.liveliness_map.get(x).unwrap().use_num)
		.sum::<usize>();
	min_uses = min(
		dag.nodes[instr_id]
			.borrow()
			.instr
			.get_riscv_write()
			.iter()
			.map(|x| state.liveliness_map.get(x).unwrap().use_num)
			.min()
			.unwrap_or(0),
		min_uses,
	);
	let mut end_live_score = (sum_uses as i32) * SUM_MIN_RATIO;
	end_live_score += min_uses as i32;
	// 判断对后继的影响
	let succ_sum = dag.nodes[instr_id]
		.borrow()
		.succ
		.iter()
		.map(|x| {
			x.borrow()
				.instr
				.get_riscv_read()
				.iter()
				.map(|y| state.liveliness_map.get(y).unwrap().use_num)
				.sum::<usize>()
		})
		.sum::<usize>()
		+ dag.nodes[instr_id]
			.borrow()
			.succ
			.iter()
			.map(|x| {
				x.borrow()
					.instr
					.get_riscv_write()
					.iter()
					.map(|y| state.liveliness_map.get(y).unwrap().use_num)
					.sum::<usize>()
			})
			.sum::<usize>();
	let succ_min = min(
		dag.nodes[instr_id]
			.borrow()
			.succ
			.iter()
			.map(|x| {
				x.borrow()
					.instr
					.get_riscv_read()
					.iter()
					.map(|y| state.liveliness_map.get(y).unwrap().use_num)
					.min()
					.unwrap_or(0)
			})
			.min()
			.unwrap_or(0),
		dag.nodes[instr_id]
			.borrow()
			.succ
			.iter()
			.map(|x| {
				x.borrow()
					.instr
					.get_riscv_write()
					.iter()
					.map(|y| state.liveliness_map.get(y).unwrap().use_num)
					.min()
					.unwrap_or(0)
			})
			.min()
			.unwrap_or(0),
	);
	let mut succ_score = (succ_sum as i32) * SUM_MIN_RATIO;
	succ_score += succ_min as i32;
	score = score * REDUCE_LIVE
		+ alloc_score * ADD_ALLOCATABLES
		+ end_live_score * NEAR_END
		+ succ_score * REDUCE_SUB;
	return score;
}
#[derive(Clone)]
struct State {
	instrs: RiscvInstrSet,
	score: i32,
	indegs: HashMap<usize, usize>, // 把节点的 id 映射到入度
	liveliness_map: HashMap<RiscvTemp, Liveliness>,
}
pub fn instr_schedule_by_dag(
	dag: InstrDag,
	liveliness_map: HashMap<RiscvTemp, Liveliness>,
) -> Result<RiscvInstrSet, SysycError> {
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
		liveliness_map: liveliness_map,
	});
	let depth = dag.nodes.len(); // bfs 深度已知，是所需要调度的指令总数
	for _i in 0..depth {
		let real_cnt = states.len();
		for _i in 0..real_cnt {
			let state = states.pop_front().unwrap();
			let state_indeg = state.indegs.clone();
			let allocatables: Vec<_> = state_indeg
				.into_iter()
				.filter(|(_k, v)| *v == 0)
				.map(|(k, _)| k)
				.collect();
			for i in allocatables.iter() {
				let mut new_state = state.clone();
				new_state.instrs.push(dag.nodes[*i].borrow().instr.clone());
				new_state.score += punishment(dag.clone(), &mut new_state, *i);
				// decl the use in new_state's liveliness_map
				for i in dag.nodes[*i].borrow().instr.get_riscv_read().iter() {
					new_state.liveliness_map.get_mut(i).unwrap().use_num -= 1;
				}
				for succ in dag.nodes[*i].borrow().succ.iter() {
					let mut new_indeg = new_state.indegs.clone();
					new_indeg.insert(
						succ.borrow().id,
						new_indeg.get(&succ.borrow().id).unwrap() - 1,
					);
					new_state.indegs = new_indeg;
				}
				states.push_back(new_state);
			}
		}
		if states.len() > BFS_STATE_THRESHOLD {
			states.make_contiguous().sort_by(|a, b| a.score.cmp(&b.score));
			states.truncate(BFS_STATE_THRESHOLD);
		}
	}
	return Ok(states.pop_front().unwrap().instrs);
}
