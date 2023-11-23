use std::fmt::Display;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum VarType {
	I32,
	F32,
	I32Ptr,
	F32Ptr,
	Void,
}

impl VarType {
	// convert ptr to value
	pub fn to_value(&self) -> Self {
		match self {
			Self::I32Ptr => Self::I32,
			Self::F32Ptr => Self::F32,
			_ => *self,
		}
	}
}

impl Display for VarType {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let type_str = match self {
			Self::I32 => "i32",
			Self::I32Ptr => "i32*",
			Self::F32 => "f32",
			Self::F32Ptr => "f32*",
			Self::Void => "void",
		};
		write!(f, "{}", type_str)
	}
}
