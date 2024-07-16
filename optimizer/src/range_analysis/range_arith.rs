use llvm::ArithOp;

use super::range::{Range, RangeItem};

fn add(a: &Range, b: &Range) -> Range {
	if a.contra || b.contra {
		return Range::contra();
	}

	fn checked_add(i: i32, j: i32) -> Option<RangeItem> {
		let (result, overflow) = i.overflowing_add(j);
		if overflow {
			None
		} else {
			Some(RangeItem::IntValue(result))
		}
	}

	fn checked_add_item(i: &RangeItem, j: &RangeItem) -> Option<RangeItem> {
		if let (RangeItem::IntValue(i), RangeItem::IntValue(j)) = (i, j) {
			checked_add(*i, *j)
		} else {
			None
		}
	}

	let new_low = match (&a.lower, &b.lower) {
		(Some(i), Some(j)) => checked_add_item(&i, &j),
		_ => None,
	};

	let new_upper = match (&a.upper, &b.upper) {
		(Some(i), Some(j)) => checked_add_item(&i, &j),
		_ => None,
	};

	Range {
		lower: new_low,
		upper: new_upper,
		contra: false,
	}
}



fn sub(a: &Range, b: &Range) -> Range {
	if a.contra || b.contra {
		return Range::contra();
	}

	fn checked_sub(i: i32, j: i32) -> Option<RangeItem> {
		let (result, overflow) = i.overflowing_sub(j);
		if overflow {
			None
		} else {
			Some(RangeItem::IntValue(result))
		}
	}

	fn checked_sub_item(i: &RangeItem, j: &RangeItem) -> Option<RangeItem> {
		if let (RangeItem::IntValue(i), RangeItem::IntValue(j)) = (i, j) {
			checked_sub(*i, *j)
		} else {
			None
		}
	}

	let new_low = match (&a.lower, &b.upper) {
		(Some(i), Some(j)) => checked_sub_item(&i, &j),
		_ => None,
	};

	let new_upper = match (&a.upper, &b.lower) {
		(Some(i), Some(j)) => checked_sub_item(&i, &j),
		_ => None,
	};

	Range {
		lower: new_low,
		upper: new_upper,
		contra: false,
	}
}









pub fn range_calculate(op: &ArithOp, srcs: Vec<&Range>) -> Range {
	match op {
			ArithOp::Add => add(srcs[0], srcs[1]),
			ArithOp::Sub => sub(srcs[0], srcs[1]),
		ArithOp::Div => todo!(),
		ArithOp::Mul => todo!(),
		ArithOp::Rem => todo!(),
		ArithOp::Fadd => todo!(),
		ArithOp::Fsub => todo!(),
		ArithOp::Fdiv => todo!(),
		ArithOp::Fmul => todo!(),
		_ => Range::inf(),
	}
}
