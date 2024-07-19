use super::range::Range;
use llvm::CompOp;

pub fn comp_must_never(
	op: &CompOp,
	range1: &Range,
	range2: &Range,
) -> Option<bool> {
	if must(op, range1, range2) {
		Some(true)
	} else if never(op, range1, range2) {
		Some(false)
	} else {
		None
	}
}

fn must(op: &CompOp, range1: &Range, range2: &Range) -> bool {
	match op {
		CompOp::EQ | CompOp::OEQ => {
			range1.lower == range1.upper
				&& range2.lower == range2.upper
				&& range1.lower == range2.upper
		}
		CompOp::NE | CompOp::ONE => {
			range1.upper < range2.lower || range2.upper < range2.lower
		}
		CompOp::SGT | CompOp::OGT => range1.lower > range2.upper,
		CompOp::SGE | CompOp::OGE => range1.lower >= range2.upper,
		CompOp::SLT | CompOp::OLT => range1.upper < range2.lower,
		CompOp::SLE | CompOp::OLE => range1.upper <= range2.lower,
	}
}

fn never(op: &CompOp, range1: &Range, range2: &Range) -> bool {
	match op {
		CompOp::EQ | CompOp::OEQ => {
			range1.upper < range2.lower || range2.upper < range2.lower
		}
		CompOp::NE | CompOp::ONE => {
			range1.lower == range1.upper
				&& range2.lower == range2.upper
				&& range1.lower == range2.upper
		}
		CompOp::SGT | CompOp::OGT => range1.upper <= range2.lower,
		CompOp::SGE | CompOp::OGE => range1.upper < range2.lower,
		CompOp::SLT | CompOp::OLT => range1.lower >= range2.upper,
		CompOp::SLE | CompOp::OLE => range1.lower > range2.upper,
	}
}
