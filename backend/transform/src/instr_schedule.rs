use std::{
	cmp::max,
	collections::{HashMap, HashSet, VecDeque},
};

use crate::instrdag::InstrDag;
use instruction::RiscvInstrSet;
use utils::{SysycError, BFS_STATE_THRESHOLD};

// 当前惩罚策略：在指令为 instrs 的情况下，在运行每一条指令期间活跃的最大寄存器数目
// 接受参数：dag:在最初始图上删除和 instr 相关边所得到的图，instrs:当前的指令序列
fn punishment(dag: InstrDag, instrs: RiscvInstrSet) -> i32 {
	// 认为一个变量的活跃范围是基本块的开头一直到 last use 这条指令的时候 ?
	// todo 总感觉依赖于dag 应该有算法，但是现在有点想不出来
	return 0;
}
#[derive(Clone)]
struct State {
	instrs: RiscvInstrSet,
	score: i32,
	indegs: HashMap<usize, usize>, // 把节点的 id 映射到入度
}
pub fn instr_schedule_by_dag(
	dag: InstrDag,
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
	});
	let depth = dag.nodes.len(); // bfs 深度已知，是所需要调度的指令总数
	for i in 0..depth {
		let mut real_cnt = states.len();
		for i in 0..real_cnt {
			let state = states.pop_front().unwrap();
			let state_indeg = state.indegs.clone();
			let mut allocatables: Vec<_> = state_indeg
				.into_iter()
				.filter(|(k, v)| *v == 0)
				.map(|(k, _)| k)
				.collect();
			for i in allocatables.iter() {
				let mut new_state = state.clone();
				new_state.instrs.push(dag.nodes[*i].borrow().instr.clone());
				new_state.score = punishment(dag.clone(), new_state.instrs.clone());
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
