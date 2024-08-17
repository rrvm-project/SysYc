use instr_schedule::instr_schedule_by_dag;
use instrdag::InstrDag;
use instruction::riscv::value::RiscvTemp;
use rrvm::{
	program::{RiscvFunc, RiscvProgram},
	RiscvCFG, RiscvNode,
};
use std::collections::{HashMap, HashSet};
use utils::{errors::Result, SCHEDULE_THRESHOLD};
pub mod instr_schedule;
pub mod instrdag;

pub fn instr_schedule_program(program: &mut RiscvProgram) {
	for func in program.funcs.iter_mut() {
		instr_schedule(func);
	}
}
pub fn instr_schedule(func: &mut RiscvFunc) {
	func.cfg.clear_data_flow();
	func.cfg.analysis();
	let mut new_blocks = Vec::new();
	for node in func.cfg.blocks.iter() {
		let liveins = node
			.borrow()
			.live_in
			.iter()
			.map(|x| RiscvTemp::VirtReg(*x))
			.collect::<HashSet<_>>();
		let liveouts = node
			.borrow()
			.live_out
			.iter()
			.map(|x| RiscvTemp::VirtReg(*x))
			.collect::<HashSet<_>>();
		let nodes = instr_schedule_block(node, &liveins, &liveouts).unwrap();
		new_blocks.extend(nodes);
	}
	func.cfg = RiscvCFG { blocks: new_blocks };
}
pub fn instr_schedule_block(
	riscv_node: &RiscvNode,
	live_ins: &HashSet<RiscvTemp>,
	live_outs: &HashSet<RiscvTemp>,
) -> Result<Vec<RiscvNode>> {
	if riscv_node.borrow().instrs.len() >= SCHEDULE_THRESHOLD {
		return Ok(vec![riscv_node.clone()]);
	}
	transform_basic_block_by_pipelining(riscv_node, live_ins, live_outs)
		.map(|v| vec![v])
}
fn transform_basic_block_by_pipelining(
	node: &RiscvNode,
	live_in: &HashSet<RiscvTemp>,
	live_out: &HashSet<RiscvTemp>,
) -> Result<RiscvNode> {
	let mut instr_dag = InstrDag::new(node)?;
	let liveliness_map = get_liveliness_map(&instr_dag, live_in, live_out);
	instr_dag.assign_nodes();
	node.borrow_mut().instrs = instr_schedule_by_dag(instr_dag, liveliness_map)?;
	Ok(node.clone())
}
#[derive(Clone, Debug)]
pub struct Liveliness {
	is_livein: bool,
	is_liveout: bool,
	use_num: usize,
}
fn get_liveliness_map(
	node: &InstrDag,
	live_in: &HashSet<RiscvTemp>,
	live_out: &HashSet<RiscvTemp>,
) -> HashMap<RiscvTemp, Liveliness> {
	let mut map = HashMap::new();
	let mut call_reads = node.call_reads.clone();
	call_reads.reverse();
	let mut call_writes = node.call_writes.clone();
	call_writes.reverse();
	// 它这里要求是正序遍历，所以遍历次序是和 node 的顺序反的，需要 iter.rev(),同样，call_reads,call_writes 也要reverse再pop
	for instrnode in node.nodes.iter().rev() {
		let instr = &instrnode.borrow().instr;
		if !instr.is_call() {
			for tmp in instr.get_riscv_read().iter() {
				map
					.entry(*tmp)
					.or_insert(Liveliness {
						is_livein: false,
						is_liveout: false,
						use_num: 0,
					})
					.use_num += 1;
			}
			for tmp in instr.get_riscv_write().iter() {
				map.entry(*tmp).or_insert(Liveliness {
					is_livein: false,
					is_liveout: false,
					use_num: 0,
				});
			}
		} else {
			let call_read = call_reads.pop().unwrap();
			for tmp in call_read.iter() {
				map
					.entry(*tmp)
					.or_insert(Liveliness {
						is_livein: false,
						is_liveout: false,
						use_num: 0,
					})
					.use_num += 1;
			}
			let call_write = call_writes.pop().unwrap();
			for tmp in call_write.iter() {
				map.entry(*tmp).or_insert(Liveliness {
					is_livein: false,
					is_liveout: false,
					use_num: 0,
				});
			}
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
