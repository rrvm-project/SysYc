use llvm::ArithOp;

use super::range::Range;

pub fn range_calculate(op: &ArithOp, srcs: Vec<&Range>) -> Range {
	match op {
		ArithOp::Add | ArithOp::Fadd => srcs[0].add(srcs[1]),
		ArithOp::Div | ArithOp::Fdiv => srcs[0].div(srcs[1]),
		ArithOp::Mul | ArithOp::Fmul => srcs[0].mul(srcs[1]),
		ArithOp::Rem => srcs[0].rem(srcs[1]),
		ArithOp::Sub | ArithOp::Fsub => srcs[0].sub(srcs[1]),
		_ => Range::inf(),
	}
}
