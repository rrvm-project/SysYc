use llvm::llvmop::Value;

#[derive(Clone)]
pub enum Attr {
	FuncSymbol(usize),
	VarSymbol(usize),
	UIntValue(usize),
	IntValue(i32),
	// used in llvmgen
	Value(Value),
}

pub trait Attrs {
	fn set_attr(&mut self, name: &str, attr: Attr);
	fn get_attr(&self, name: &str) -> Option<&Attr>;
}
