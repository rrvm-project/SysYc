use crate::{llvmop::Value, LlvmTemp};

pub fn unwrap_values(arr: Vec<&Value>) -> Vec<LlvmTemp> {
	arr.into_iter().flat_map(|v| v.unwrap_temp()).collect()
}
