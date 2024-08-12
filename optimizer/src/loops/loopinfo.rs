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
	pub single_exit: LlvmNode,
	pub cmp: LlvmTemp,
	pub comp_op: CompOp,
	pub step: Value,
	pub begin: Value,
	pub end: Value,
}
