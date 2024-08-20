use std::collections::HashMap;

use llvm::LlvmTemp;
use rrvm::{program::LlvmFunc, rrvm_loop::LoopPtr, LlvmNode};

use super::{indvar::IndVar, loopinfo::LoopInfo, temp_graph::TempGraph};
pub struct LoopData {
	// 前两项建议每次使用前重建，后四项建议动态维护
	// 从自己指向自己的 use
	pub temp_graph: TempGraph,
	// 每个变量在哪个基本块中被定义
	pub def_map: HashMap<LlvmTemp, LlvmNode>,
	// 每个 basicblock 属于哪个循环
	pub loop_map: HashMap<i32, LoopPtr>,
	// 循环树的根
	pub root_loop: LoopPtr,
	// loop id to loopinfo
	// 仅能确定循环次数的 loop 才有 LoopInfo
	pub loop_infos: HashMap<i32, LoopInfo>,
	// Temp to IndVar
	// Temp 仅能相对于它的定义所在的 Loop 来说是 IndVar
	pub indvars: HashMap<LlvmTemp, IndVar>,
	pub scc_map: HashMap<LlvmTemp, Vec<LlvmTemp>>,
}

impl LoopData {
	pub fn new(func: &mut LlvmFunc) -> Self {
		let def_map = Self::build_def_map(func);
		let temp_graph = Self::build_graph(func);
		let (root_loop, loop_map) = func.cfg.loop_analysis();
		Self {
			temp_graph,
			loop_map,
			def_map,
			root_loop,
			loop_infos: HashMap::new(),
			indvars: HashMap::new(),
			scc_map: HashMap::new(),
		}
	}

	pub fn build_def_map(func: &LlvmFunc) -> HashMap<LlvmTemp, LlvmNode> {
		let mut def_map = HashMap::new();
		for bb in func.cfg.blocks.iter() {
			for inst in bb.borrow().phi_instrs.iter() {
				def_map.insert(inst.target.clone(), bb.clone());
			}
			for inst in bb.borrow().instrs.iter() {
				if let Some(target) = inst.get_write() {
					def_map.insert(target.clone(), bb.clone());
				}
			}
		}
		def_map
	}
	pub fn rebuild(&mut self, func: &mut LlvmFunc) {
		self.def_map = Self::build_def_map(func);
		self.temp_graph = Self::build_graph(func);
	}
}
