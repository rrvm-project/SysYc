use utils::ValueItem::{self, *};

use crate::Value;

pub fn to_data(value: Value) -> Vec<ValueItem> {
	match value {
		Value::Int(0) => vec![ValueItem::Zero(4)],
		Value::Int(v) => vec![ValueItem::Word(v as u32)],
		Value::Float(v) => vec![ValueItem::Word(v.to_bits())],
		Value::Array((_index, arr)) => {
			arr.into_iter().flat_map(to_data).fold(Vec::new(), |mut acc, v| {
				if let (Some(Zero(last)), Zero(now)) = (acc.last_mut(), &v) {
					*last += *now;
				} else {
					acc.push(v);
				}
				acc
			})
		}
	}
}
