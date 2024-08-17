use llvm::{CompOp, LlvmTemp, Value};
use rrvm::LlvmNode;

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
	pub preheader: LlvmNode,
	pub header: LlvmNode,
	// 它会是 dedicated exit, 也即 它的前驱只有循环中的块
	pub single_exit: LlvmNode,
	pub cmp: LlvmTemp,
	pub comp_op: CompOp,
	pub step: Value,
	pub begin: Value,
	pub end: Value,
}

impl LoopInfo {
	pub fn has_const_loop_cnt(&self) -> Option<i32> {
		if let (Value::Int(begin), Value::Int(end), Value::Int(step)) =
			(&self.begin, &self.end, &self.step)
		{
			match self.comp_op {
				CompOp::SLT => {
					let mut full_cnt = (end - begin + step - 1) / step;
					if begin >= end {
						full_cnt = 0;
					}
					Some(full_cnt)
				}
				CompOp::SLE => {
					let mut full_cnt = (end - begin + step) / step;
					if begin > end {
						full_cnt = 0;
					}
					Some(full_cnt)
				}
				_ => unreachable!(),
			}
		} else {
			None
		}
	}
}
