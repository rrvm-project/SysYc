use std::{borrow::Borrow, collections::HashMap, hash::Hash};

#[derive(Debug)]
pub struct StackHashMap<K, V> {
	stack: Vec<HashMap<K, V>>,
}

impl<K, V> StackHashMap<K, V>
where
	K: Hash + PartialEq + Eq,
{
	pub fn new() -> Self {
		StackHashMap { stack: Vec::new() }
	}

	pub fn push(&mut self) {
		self.stack.push(HashMap::new())
	}

	pub fn pop(&mut self) {
		self.stack.pop();
	}

	pub fn insert(&mut self, k: K, v: V) -> Option<V> {
		self.stack.last_mut().unwrap().insert(k, v)
	}

	pub fn get<Q>(&self, k: &Q) -> Option<&V>
	where
		Q: ?Sized,
		K: Borrow<Q>,
		Q: Hash + Eq,
	{
		for item in &self.stack {
			if let Some(reuslt) = item.get(k) {
				return Some(reuslt);
			}
		}
		None
	}
}
