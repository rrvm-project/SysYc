use std::fmt::Display;

#[derive(Clone, PartialEq, Eq)]
pub enum VarType {
	I32,
	F32,
	I32Ptr,
	F32Ptr,
}

impl Display for VarType {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let type_str = match self {
			Self::I32 => "i32",
			Self::I32Ptr => "i32*",
			Self::F32 => "f32",
			Self::F32Ptr => "f32*",
		};
		write!(f, "{}", type_str)
	}
}
