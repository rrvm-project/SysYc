use utils::SysycError::{self, *};

pub fn negative_dim_length() -> SysycError {
	TypeError("The size of array must be positive".to_string())
}

pub fn array_dims_error() -> SysycError {
	TypeError("The length of array must be constant integer".to_string())
}

pub fn uninitialized(ident: &str) -> SysycError {
	SemanticError(format!("uninitialized 'const {}'", ident))
}

pub fn initialize_by_none(ident: &str) -> SysycError {
	SemanticError(format!(
		"'const {}' is initialized by non-const expression",
		ident
	))
}
