use std::collections::HashMap;

use llvm::{CompOp, LlvmTemp, Value};
use rrvm::LlvmNode;

use super::indvar::IndVar;

/// entry:
///   br label %B1
/// B1:
///   %1 = phi i32 [0, label %entry], [%5, label %B9]
///   %2 = phi i32 [0, label %entry], [%4, label %B9]
///   %3 = icmp slt i32 %1, 100
///   br i32 %3, label %B2, label %B3
/// B2:
///   %4 = add i32 %2, %1
///   %5 = add i32 %1, 1
///   br label %B1
/// B3:
///   ret i32 %2
#[derive(Clone)]
pub struct LoopInfo {
	pub indvars: HashMap<LlvmTemp, IndVar>,
	/// %3
	pub branch_temp: LlvmTemp,
	/// slt
	pub comp_op: CompOp,
	/// 100
	pub end: Value,
	/// %1
	pub loop_cond_temp: LlvmTemp,
	/// 循环次数，如果是一个 temp, 则计算这个 temp 的语句会被插入 preheader
	pub loop_cnt: Value,
	pub header: LlvmNode,
	pub preheader: LlvmNode,
	pub single_exit: LlvmNode,
}

#[allow(unused)]
impl LoopInfo {
	pub fn get_start(&self) -> Value {
		self.indvars[&self.loop_cond_temp].base.clone()
	}
	pub fn get_end(&self) -> Value {
		self.end.clone()
	}
	pub fn get_step(&self) -> Value {
		// 只考虑 scale 为 1 的情况
		assert!(self.indvars[&self.loop_cond_temp].scale == Value::Int(1));
		self.indvars[&self.loop_cond_temp].step.clone()
	}
}