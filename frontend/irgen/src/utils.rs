use crate::{irgen::LlvmIrGen, IRVALUE, SYMBOL, VALUE};
use ast::tree::Node;
use attr::Attr;
use llvm::llvmop::Value;
use rrvm_symbol::VarSymbol;
use utils::{Result, SysycError};

// 如果有编译器常量则优先返回，否则返回 IRValue

impl LlvmIrGen {
	// 如果挂上的 IRValue 是地址，增加一条 load 指令取出地址
	pub fn get_value(&mut self, node: &Node) -> Result<Value> {
		if let Some(Attr::Value(const_value)) = node.get_attr(VALUE) {
			match const_value {
				value::Value::Int(v) => Ok(Value::Int(*v)),
				value::Value::Float(v) => Ok(Value::Float(*v)),
				_ => Err(SysycError::LlvmSyntaxError(format!(
					"Compile const value should not be {:?}",
					const_value
				))),
			}
		} else if let Some(Attr::IRValue(ir_value)) = node.get_attr(IRVALUE) {
			if ir_value.is_ptr() {
				Ok(Value::Temp(
					self.funcemitter.as_mut().unwrap().visit_load_instr(ir_value.clone()),
				))
			} else {
				Ok(ir_value.clone())
			}
		} else {
			Err(SysycError::LlvmNoValueError(
				"node has no value".to_string(),
			))
		}
	}
	// 用于左操作数，允许直接取出地址
	pub fn get_lhs_value(&mut self, node: &Node) -> Result<Value> {
		if let Some(Attr::IRValue(ir_value)) = node.get_attr(IRVALUE) {
			Ok(ir_value.clone())
		} else {
			Err(SysycError::LlvmNoValueError("lhs has no value".to_string()))
		}
	}
	pub fn get_symbol(&mut self, node: &Node) -> Result<VarSymbol> {
		if let Some(Attr::VarSymbol(symbol)) = node.get_attr(SYMBOL) {
			Ok(symbol.clone())
		} else {
			Err(SysycError::LlvmNoSymbolError(
				"node has no symbol".to_string(),
			))
		}
	}
}

pub fn get_bool_value(v: &Value) -> Option<bool> {
	match v {
		Value::Int(v) => Some(*v != 0),
		Value::Float(v) => Some(*v != 0.0),
		_ => None,
	}
}
