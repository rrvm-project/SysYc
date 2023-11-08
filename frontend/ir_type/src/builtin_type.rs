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
		if self.dims.len() > 0 {
			for i in 0..self.dims.len() {
				if self.dims[i] != other.dims[i] {
					if i == 0 {
						if self.dims[i] != 0 && self.dims[i] != 0 {
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
	fn dim_length(&self) -> usize {
		self.dims.len()
	}

	fn new_scalar(base_type: BaseType, is_const: bool) -> Self {
		IRType {
			base_type: base_type,
			dims: vec![],
			is_const: is_const,
		}
	}

	fn new_array(base_type: BaseType, is_const: bool, dims: Vec<usize>) -> Self {
		IRType {
			base_type: base_type,
			dims: dims,
			is_const: is_const,
		}
	}

	fn size(&self) -> usize {
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
