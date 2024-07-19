use super::range::{Range, RangeItem};

pub trait WidenOp {
	//return true iff the range changed
	fn widen(&self, original: &mut Option<Range>, evaluation: Range) -> bool;
}

pub struct SimpleWidenOperator;

impl WidenOp for SimpleWidenOperator {
	fn widen(&self, original: &mut Option<Range>, evaluation: Range) -> bool {
		// dbg!((&original, &evaluation));

		// dbg!(&original);
		if let Some(original) = original {
			let mut change_flag = false;
			if original.lower > evaluation.lower {
				original.lower = RangeItem::NegInf;
				change_flag = true;
			}
			if original.upper < evaluation.upper {
				original.upper = RangeItem::PosInf;
				change_flag = true;
			}
			change_flag
		} else {
			*original = Some(evaluation);
			true
		}
	}
}
