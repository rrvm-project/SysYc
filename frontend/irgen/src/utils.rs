use crate::{IRVALUE, VALUE};
use ast::tree::Node;
use attr::Attr;
use llvm::llvmop::Value;
use utils::{Result, SysycError};

// 如果有编译器常量则优先返回，否则返回 IRValue
pub fn get_value(node: &Node) -> Result<Value> {
	if let Some(Attr::Value(const_value)) = node.get_attr(VALUE) {
		match const_value {
			value::Value::Int(v) => Ok(llvm::llvmop::Value::Int(*v)),
			value::Value::Float(v) => Ok(llvm::llvmop::Value::Float(*v)),
			_ => Err(SysycError::LlvmSyntexError(format!(
				"Compile const value should not be {:?}",
				const_value
			))),
		}
	} else if let Some(Attr::IRValue(ir_value)) = node.get_attr(IRVALUE) {
		Ok(ir_value.clone())
	} else {
		Err(SysycError::LlvmSyntexError("node has no value".to_string()))
	}
}
