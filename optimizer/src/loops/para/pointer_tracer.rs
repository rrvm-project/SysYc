use std::collections::{HashMap, HashSet};

use llvm::LlvmTemp;

pub struct PointerTracer {
	ptr_set: HashMap<LlvmTemp, u32>,
	named: HashMap<String, u32>,
	read: HashSet<u32>,
	write: HashSet<u32>,
	last: u32,
}

impl PointerTracer {
	pub fn new() -> Self {
		Self {
			ptr_set: HashMap::new(),
			last: 0,
			read: HashSet::new(),
			write: HashSet::new(),
			named: HashMap::new(),
		}
	}
	pub fn get(&mut self, ptr: &LlvmTemp) -> u32 {
		*self.ptr_set.entry(ptr.clone()).or_insert_with(|| 0)
	}

	pub fn create(&mut self, ptr: &LlvmTemp) -> u32 {
		*self.ptr_set.entry(ptr.clone()).or_insert_with(|| {
			self.last += 1;
			self.last
		})
	}

	pub fn name(&mut self, ptr: &LlvmTemp, ident: &String) -> u32 {
		// *self.named.entry(ident.clone()).or_insert_with(||self.create(ptr))
		if let Some(id) = self.named.get(ident) {
			*id
		} else {
			let id = self.create(ptr);
			self.named.insert(ident.clone(), id);
			id
		}
	}

	pub fn link(&mut self, src: &LlvmTemp, dst: &LlvmTemp) -> u32 {
		let c = self.get(dst);
		self.ptr_set.insert(src.clone(), c);
		c
	}

	pub fn clear(&mut self) -> (HashSet<u32>, HashSet<u32>) {
		let read = std::mem::take(&mut self.read);
		let write = std::mem::take(&mut self.write);
		(read, write)
	}

	pub fn merge(&mut self, read: HashSet<u32>, write: HashSet<u32>) -> bool {
		self.read.extend(read);
		self.write.extend(write);
		!self.read.is_disjoint(&self.write)
	}

	pub fn read(&mut self, a: &LlvmTemp) -> bool {
		let c = self.get(a);
		self.read.insert(c);
		self.write.contains(&c) || c == 0
	}

	pub fn write(&mut self, a: &LlvmTemp) -> bool {
		let c = self.get(a);
		self.write.insert(c);
		self.read.contains(&c) || c == 0
	}
}
