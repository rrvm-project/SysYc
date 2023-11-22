use thiserror::Error;

pub use SysycError::*;

#[derive(Error, Debug)]
pub enum SysycError {
	#[error("{0}")]
	LexError(String),
	#[error("Syntax Error : {0}")]
	SyntaxError(String),
	#[error("Type Error : {0}")]
	TypeError(String),
	#[error("System error: {0}")]
	SystemError(String),
	#[error("Fatal error: {0}")]
	FatalError(String),
	#[error("Llvm syntax error: {0}")]
	LlvmSyntaxError(String),
	#[error("Llvm no value error: {0}")]
	LlvmNoValueError(String),
	#[error("Llvm no symbol error: {0}")]
	LlvmNoSymbolError(String),
	#[error("Riscv generating error: {0}")]
	RiscvGenError(String),
}

pub type Result<T, E = SysycError> = core::result::Result<T, E>;

pub fn map_sys_err(e: std::io::Error) -> SysycError {
	SystemError(e.to_string())
}
