// #[derive(Debug, Clone, PartialEq, Eq)]
// pub struct BuiltinType {
//     pub name: String,
// }

// pub trait ir_type {
//     fn is_base(&self) -> bool;
//     fn is_array(&self) -> bool;
//     fn indexed(&self) -> Option<&Self>;
//     fn size(&self) -> usize;
//     fn name(&self) -> String;
//     fn dims(&self) -> Vec<u32>;
// }

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum BaseType {
	Int,
	Float,
	Void,
}

#[derive(Debug, Clone, Eq)]
pub struct IRType {
	pub base_type: BaseType,
	pub dims: Vec<usize>,
	pub is_const: bool,
}

impl PartialEq for IRType {
	fn eq(&self, other: &Self) -> bool {
		if self.base_type != other.base_type {
			return false;
		}
		if self.dims.len() != other.dims.len() {
			return false;
		}
		if !self.dims.is_empty() {
			for i in 0..self.dims.len() {
				if self.dims[i] != other.dims[i] {
					if i == 0 {
						if self.dims[i] != 0 && other.dims[i] != 0 {
							return false;
						}
					} else {
						return false;
					}
				}
			}
		}
		//Ignore Comparing Const or not
		true
	}
}

impl IRType {
	pub fn dim_length(&self) -> usize {
		self.dims.len()
	}

	pub fn get_scalar(base_type: BaseType, is_const: bool) -> Self {
		IRType {
			base_type,
			dims: vec![],
			is_const,
		}
	}

	pub fn is_array(&self) -> bool {
		self.dim_length() > 0
	}

	pub fn is_scalar(&self) -> bool {
		self.dim_length() == 0
	}

	pub fn get_index(&self, a: &Vec<usize>) -> usize {
		let mut ans: usize = if !a.is_empty() { a[0] } else { 0 };

		let length = self.dim_length();
		for i in 1..length {
			ans *= self.dims[i];
			ans += if i < a.len() { a[i] } else { 0 };
		}
		ans
	}

	pub fn size(&self) -> usize {
		let mut i = match self.base_type {
			BaseType::Int => 4,
			BaseType::Float => 4,
			_ => unreachable!(),
		};

		for size in &self.dims {
			i *= size
		}
		i
	}
}
