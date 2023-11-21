use value::{BType, Value};

use std::collections::HashMap;

use utils::Result;

fn flat_to_indexes(flat: usize, dim_list: &Vec<usize>) -> Vec<usize> {
	let mut alignment = vec![];
	alignment.push(1);
	let len = dim_list.len();

	for i in 1..len {
		let current_size = dim_list[len - i];
		alignment.push(alignment[i - 1] * current_size);
	}

	let mut result = vec![];
	let mut remain = flat;
	for i in 0..len {
		result.push(remain / alignment[len - i - 1]);
		remain %= alignment[len - i - 1];
	}

	result
}

pub fn get_value_for_calc(
	tp: BType,
	dim_list: &Vec<usize>,
	value_map: &HashMap<usize, Value>,
) -> Result<Value> {
	let mut value_i32 = HashMap::new();
	let mut value_f32 = HashMap::new();

	let length = &dim_list.len();

	for (key, value) in value_map {
		match tp {
			BType::Int => {
				value_i32
					.insert(flat_to_indexes(*key, dim_list), value.get_i32_value()?);
			}
			BType::Float => {
				value_f32
					.insert(flat_to_indexes(*key, dim_list), value.get_f32_value()?);
			}
		};
	}

	match tp {
		BType::Int => Ok(Value::IntPtr((vec![], (*length, value_i32)))),
		BType::Float => Ok(Value::FloatPtr((vec![], (*length, value_f32)))),
	}
}

pub fn get_zero(tp: BType) -> Value {
	match tp {
		BType::Int => Value::Int(0),
		BType::Float => Value::Float(0.0),
	}
}

// mod tests {
// 	use super::flat_to_indexes;
// 	#[test]
// 	fn t() {
// 		let a: usize = 115123;
// 		let dim = vec![10, 100, 1000];
// 		dbg!(flat_to_indexes(a, &dim));
// 	}
// }
