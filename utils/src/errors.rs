use thiserror::Error;

pub use SysycError::*;

#[derive(Error, Debug)]
pub enum SysycError {
	#[error("{0}")]
	LexError(String),
	#[error("Syntax Error: {0}")]
	SyntaxError(String),
	#[error("Type Error: {0}")]
	TypeError(String),
	#[error("Semantic Error: {0}")]
	SemanticError(String),
	#[error("System error: {0}")]
	SystemError(String),
	#[error("Fatal error: {0}")]
	FatalError(String),
	#[error("Llvm syntex error: {0}")]
	LlvmSyntexError(String),
	#[error("Riscv generating error: {0}")]
	RiscvGenError(String),
	#[error("Llvm generating error: {0}")]
	LlvmvGenError(String),
}

pub type Result<T, E = SysycError> = core::result::Result<T, E>;

pub fn map_sys_err(e: std::io::Error) -> SysycError {
	SystemError(e.to_string())
}
