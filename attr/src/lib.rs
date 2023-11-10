use llvm::llvmop::Value;
use ir_type::builtin_type::IRType;
use std::collections::HashMap;
use utils::SysycError;

#[derive(Debug, Clone)]
pub enum Attr {
	CompileConstValue(CompileConstValue),
	FuncSymbol(usize),
	VarSymbol(usize),
	Type(IRType),
	UIntValue(usize),
	IntValue(i32),

	// used in llvmgen
	Value(Value),
}

#[derive(Debug, Clone)]
pub enum CompileConstValue {
	Int(i32),
	Float(f32),
	IntArray(HashMap<usize, i32>),
	FloatArray(HashMap<usize, f32>),
}

impl From<i32> for CompileConstValue {
	fn from(value: i32) -> Self {
		CompileConstValue::Int(value)
	}
}

impl From<f32> for CompileConstValue {
	fn from(value: f32) -> Self {
		CompileConstValue::Float(value)
	}
}

impl CompileConstValue {
	pub fn to_i32(&self) -> Result<i32, SysycError> {
		let err = "Array can not be transformed into int value".to_string();
		match self {
			CompileConstValue::Int(v) => Ok(*v),
			CompileConstValue::Float(v) => Ok(*v as i32),
			CompileConstValue::IntArray(_) => Err(SysycError::SyntaxError(err)),
			CompileConstValue::FloatArray(_) => Err(SysycError::SyntaxError(err)),
		}
	}

	pub fn to_f32(&self) -> Result<f32, SysycError> {
		let err = "Array can not be transformed into int value".to_string();
		match self {
			CompileConstValue::Int(v) => Ok(*v as f32),
			CompileConstValue::Float(v) => Ok(*v),
			CompileConstValue::IntArray(_) => Err(SysycError::SyntaxError(err)),
			CompileConstValue::FloatArray(_) => Err(SysycError::SyntaxError(err)),
		}
	}
}

#[derive(Debug, Clone, Copy)]
pub enum InitValueItem {
	Int(i32),
	Float(f32),
	None(usize),
}

impl InitValueItem {
	pub fn to_i32(&self) -> i32 {
		match self {
			InitValueItem::Int(v) => *v,
			InitValueItem::Float(v) => *v as i32,
			InitValueItem::None(_) => {
				unreachable!("None 类型用于填充初始化列表中空白而不是表示具体的值")
			}
		}
	}

	pub fn to_f32(&self) -> f32 {
		match self {
			InitValueItem::Int(v) => *v as f32,
			InitValueItem::Float(v) => *v,
			InitValueItem::None(_) => {
				unreachable!("None 类型用于填充初始化列表中空白而不是表示具体的值")
			}
		}
	}
}

pub trait Attrs {
	fn set_attr(&mut self, name: &str, attr: Attr);
	fn get_attr(&self, name: &str) -> Option<&Attr>;
}