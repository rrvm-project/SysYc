use std::collections::HashMap;

use llvm::{LlvmTemp, LlvmTempManager};
use rrvm::{program::LlvmFunc, rrvm_loop::LoopPtr, LlvmNode};

use super::{
	indvar_optimize::IndvarOptimize, loop_simplify::LoopSimplify,
	loopinfo::LoopInfo, temp_graph::TempGraph,
};
use crate::metadata::FuncData;

pub struct LoopOptimizer<'a> {
	// 从自己指向自己的 use
	pub temp_graph: TempGraph,
	// 每个 basicblock 属于哪个循环
	pub loop_map: HashMap<i32, LoopPtr>,
	// 每个变量在哪个基本块中被定义
	pub def_map: HashMap<LlvmTemp, LlvmNode>,
	// 循环树的根
	pub root_loop: LoopPtr,
	// loop id to loopinfo
	// 仅能确定循环次数的 loop 才有 LoopInfo
	pub loop_infos: HashMap<u32, LoopInfo>,
	pub funcdata: &'a mut FuncData,
	pub temp_mgr: &'a mut LlvmTempManager,
	pub func: &'a mut LlvmFunc,
}

impl<'a: 'b, 'b> LoopOptimizer<'a> {
	pub fn new(
		func: &'a mut LlvmFunc,
		funcdata: &'a mut FuncData,
		temp_mgr: &'a mut LlvmTempManager,
	) -> Self {
		let def_map = Self::build_def_map(func);
		let temp_graph = Self::build_graph(func);
		let (root_loop, loop_map) = func.cfg.loop_analysis();
		Self {
			temp_graph,
			loop_map,
			def_map,
			root_loop,
			loop_infos: HashMap::new(),
			func,
			funcdata,
			temp_mgr,
		}
	}

	fn build_def_map(func: &LlvmFunc) -> HashMap<LlvmTemp, LlvmNode> {
		let mut def_map = HashMap::new();
		for bb in func.cfg.blocks.iter() {
			let bb_ = bb.borrow();
			for inst in bb_.phi_instrs.iter() {
				def_map.insert(inst.target.clone(), bb.clone());
			}
			for inst in bb_.instrs.iter() {
				if let Some(target) = inst.get_write() {
					def_map.insert(target.clone(), bb.clone());
				}
			}
		}
		def_map
	}

	pub fn loop_simplify(&'b mut self) -> LoopSimplify<'a, 'b> {
		LoopSimplify::new(self)
	}

	pub fn indvar_optimze(&'b mut self) -> IndvarOptimize<'a, 'b> {
		IndvarOptimize::new(self)
	}
}
