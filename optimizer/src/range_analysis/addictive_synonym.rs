use std::{
	collections::HashMap,
	hash::Hash,
	ops::{Add, Sub},
};

use llvm::{LlvmTemp, Value};

#[derive(Debug)]
struct AddictiveSynonym<
	T: Copy + Add<Output = T> + Sub<Output = T> + Default,
	K: Eq + Hash + Clone + std::fmt::Debug,
> {
	next: usize,
	catagory: HashMap<K, usize>,
	offsets: HashMap<usize, (K, HashMap<K, T>)>,
}

impl<
		T: Copy + Add<Output = T> + Sub<Output = T> + Default,
		K: Eq + Hash + Clone + std::fmt::Debug,
	> AddictiveSynonym<T, K>
{
	pub fn new() -> Self {
		Self {
			next: 0,
			catagory: HashMap::new(),
			offsets: HashMap::new(),
		}
	}

	pub fn insert(&mut self, src: &K, dst: &K, diff: T) {
		let id = if let Some(id) = self.catagory.get(src) {
			*id
		} else {
			let id = self.next;
			self.next += 1;
			self.catagory.insert(src.clone(), id);
			let mut new_map = HashMap::new();
			new_map.insert(src.clone(), Default::default());
			self.offsets.insert(id, (src.clone(), new_map));
			id
		};
		self.catagory.insert(dst.clone(), id);
		let offsets = self.offsets.get_mut(&id).unwrap();

		let value = diff + *offsets.1.get(src).unwrap();

		offsets.1.insert(dst.clone(), value);
	}

	pub fn look_up_offset(&self, src: &K, dst: &K) -> Option<T> {
		if *src == *dst {
			return Some(Default::default());
		}
		if let (Some(id1), Some(id2)) =
			(self.catagory.get(src), self.catagory.get(dst))
		{
			if *id1 == *id2 {
				let id = id1;
				let offsets = self.offsets.get(id).unwrap();
				let v1 = offsets.1.get(src).unwrap();
				let v2 = offsets.1.get(dst).unwrap();
				Some(*v2 - *v1)
			} else {
				None
			}
		} else {
			None
		}
	}
}

#[derive(Debug)]
pub(crate) struct LlvmTempAddictiveSynonym {
	int: AddictiveSynonym<i32, LlvmTemp>,
}

impl LlvmTempAddictiveSynonym {
	pub fn new() -> Self {
		Self {
			int: AddictiveSynonym::new(),
		}
	}

	pub fn insert(&mut self, src: &LlvmTemp, dst: &LlvmTemp, diff: Value) {
		match (src.var_type, dst.var_type, &diff) {
			(llvm::VarType::I32, llvm::VarType::I32, Value::Int(i)) => {
				self.int.insert(src, dst, *i)
			}
			(llvm::VarType::F32, llvm::VarType::F32, Value::Float(_f)) => {
				//TODO
			}
			_ => unreachable!(
				"incorrect type in insert(): {:?}, {:?}, {:?}",
				src.var_type, dst.var_type, diff
			),
		}
	}

	pub fn look_up_offset(
		&self,
		src: &LlvmTemp,
		dst: &LlvmTemp,
	) -> Option<Value> {
		match (src.var_type, dst.var_type) {
			(llvm::VarType::I32, llvm::VarType::I32) => {
				self.int.look_up_offset(src, dst).map(Value::Int)
			}
			(llvm::VarType::F32, llvm::VarType::F32) => {
				//TODO
				if *src == *dst {
					Some(Value::Float(0f32))
				} else {
					None
				}
			}
			// _ => unreachable!(
			// 	"incorrect type in look_up_offset(): {:?}, {:?}",
			// 	src.var_type, dst.var_type
			// ),
			_ => None
		}
	}
}
